use crate::router::dispatch;
use crate::models::graph::WikiLinkRow;
use crate::models::node::Node;
use crate::models::obsidian::ObsidianImportResult;
use crate::path_guard::{canonicalize_path, ensure_within_root};
use crate::repositories::node_repo;
use crate::services::graph_sync;
use crate::state::AppState;
use uuid::Uuid;

#[tauri::command]
pub async fn import_obsidian_vault(
    state: tauri::State<'_, AppState>,
    vault_id: Uuid,
    root_path: String,
) -> Result<ObsidianImportResult, String> {
    dispatch(
        &state,
        "import_obsidian_vault",
        serde_json::json!({ "vault_id": vault_id, "root_path": root_path }),
    )
    .await
}

pub async fn import_obsidian_vault_impl(
    state: &AppState,
    vault_id: Uuid,
    root_path: String,
) -> Result<ObsidianImportResult, String> {
    let vault_root = sqlx::query_as::<_, (String,)>("SELECT root_path FROM vaults WHERE id = $1")
        .bind(vault_id)
        .fetch_one(&state.pg()?)
        .await
        .map_err(|e| format!("Vault not found: {}", e))?
        .0;

    let _import_root = ensure_within_root(&vault_root, &root_path).map_err(|e| e.to_string())?;
    let import_root = canonicalize_path(&root_path).map_err(|e| e.to_string())?;

    let mut files_scanned = 0i32;
    let mut nodes_created = 0i32;
    let mut links_created = 0i32;

    let md_re = regex::Regex::new(r"\[\[([^\]]+)\]\]").unwrap();

    for entry in walkdir::WalkDir::new(&import_root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
    {
        files_scanned += 1;
        let file_path = entry.path().to_string_lossy().to_string();

        let title = entry
            .path()
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let content = std::fs::read_to_string(entry.path()).unwrap_or_default();

        if node_repo::find_by_file_path(&state.pg()?, vault_id, &file_path)
            .await
            .map_err(|e| e.to_string())?
            .is_some()
        {
            continue;
        }

        let id = Uuid::new_v4();
        let wc = content.split_whitespace().count() as i32;

        let node = sqlx::query_as::<_, Node>(
            "INSERT INTO nodes (id, vault_id, title, content, content_type, file_path, word_count) VALUES ($1, $2, $3, $4, 'note', $5, $6) RETURNING id, vault_id, title, content, content_type, file_path, metadata, word_count, created_at, updated_at",
        )
        .bind(id)
        .bind(vault_id)
        .bind(&title)
        .bind(&content)
        .bind(&file_path)
        .bind(wc)
        .fetch_one(&state.pg()?)
        .await
        .map_err(|e| format!("Insert: {}", e))?;

        graph_sync::create_node_with_pg_rollback(
            &state.neo4j()?,
            &state.pg()?,
            id,
            vault_id,
            &node.title,
            &node.content_type,
        )
        .await
        .map_err(|e| e.to_string())?;

        nodes_created += 1;

        for cap in md_re.captures_iter(&content) {
            let target_title = cap[1].to_string();
            let wiki_id = Uuid::new_v4();
            sqlx::query(
                "INSERT INTO wiki_links (id, source_node_id, target_title) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING",
            )
            .bind(wiki_id)
            .bind(id)
            .bind(&target_title)
            .execute(&state.pg()?)
            .await
            .map_err(|e| format!("Wiki link insert failed: {}", e))?;

            links_created += 1;
        }
    }

    let unresolved = sqlx::query_as::<_, WikiLinkRow>(
        "SELECT id, source_node_id, target_title, resolved_node_id, context, created_at FROM wiki_links WHERE resolved_node_id IS NULL AND source_node_id IN (SELECT id FROM nodes WHERE vault_id = $1)",
    )
    .bind(vault_id)
    .fetch_all(&state.pg()?)
    .await
    .map_err(|e| format!("Failed to load unresolved wiki links: {}", e))?;

    for link in unresolved {
        let resolved = node_repo::find_by_title(&state.pg()?, vault_id, &link.target_title)
            .await
            .map_err(|e| e.to_string())?;

        if let Some(resolved) = resolved {
            sqlx::query("UPDATE wiki_links SET resolved_node_id = $1 WHERE id = $2")
                .bind(resolved.id)
                .bind(link.id)
                .execute(&state.pg()?)
                .await
                .map_err(|e| format!("Wiki link resolution update failed: {}", e))?;

            graph_sync::create_wiki_link(&state.neo4j()?, link.source_node_id, resolved.id)
                .await
                .map_err(|e| format!("Neo4j link resolution failed: {}", e))?;
        }
    }

    Ok(ObsidianImportResult {
        nodes_created,
        links_created,
        files_scanned,
    })
}
