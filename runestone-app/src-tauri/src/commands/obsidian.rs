use crate::models::graph::WikiLinkRow;
use crate::models::node::{Node, NodeIdRow};
use crate::models::obsidian::ObsidianImportResult;
use crate::state::AppState;
use uuid::Uuid;

#[tauri::command]
pub async fn import_obsidian_vault(
    state: tauri::State<'_, AppState>,
    vault_id: Uuid,
    root_path: String,
) -> Result<ObsidianImportResult, String> {
    let mut files_scanned = 0i32;
    let mut nodes_created = 0i32;
    let mut links_created = 0i32;

    let md_re = regex::Regex::new(r"\[\[([^\]]+)\]\]").unwrap();

    for entry in walkdir::WalkDir::new(&root_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "md"))
    {
        files_scanned += 1;
        let file_path = entry.path().to_string_lossy().to_string();

        let title = entry.path().file_stem().unwrap_or_default().to_string_lossy().to_string();
        let content = std::fs::read_to_string(entry.path()).unwrap_or_default();

        let existing = sqlx::query_as::<_, NodeIdRow>(
            "SELECT id FROM nodes WHERE vault_id = $1 AND file_path = $2",
        )
        .bind(vault_id)
        .bind(&file_path)
    .fetch_optional(state.pg()?)
    .await
    .map_err(|e| format!("Query: {}", e))?;

    if existing.is_some() {
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
    .fetch_one(state.pg()?)
    .await
    .map_err(|e| format!("Insert: {}", e))?;

    let _ = state.neo4j()?.run(
            neo4rs::query("CREATE (n:Node {pg_id: $pg_id, vault_id: $vault_id, title: $title, content_type: 'note'})")
                .param("pg_id", id.to_string())
                .param("vault_id", vault_id.to_string())
                .param("title", node.title.clone()),
        ).await;

        nodes_created += 1;

        for cap in md_re.captures_iter(&content) {
            let target_title = cap[1].to_string();
            let wiki_id = Uuid::new_v4();
            let _ = sqlx::query(
                "INSERT INTO wiki_links (id, source_node_id, target_title) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING",
            )
            .bind(wiki_id)
            .bind(id)
            .bind(&target_title)
            .execute(state.pg()?)
            .await;

            links_created += 1;
        }
    }

    let unresolved = sqlx::query_as::<_, WikiLinkRow>(
        "SELECT id, source_node_id, target_title, resolved_node_id, context, created_at FROM wiki_links WHERE resolved_node_id IS NULL AND source_node_id IN (SELECT id FROM nodes WHERE vault_id = $1)",
    )
    .bind(vault_id)
    .fetch_all(state.pg()?)
    .await
    .unwrap_or_default();

    for link in unresolved {
        if let Ok(Some(resolved)) = sqlx::query_as::<_, NodeIdRow>(
            "SELECT id FROM nodes WHERE vault_id = $1 AND title = $2 LIMIT 1",
        )
        .bind(vault_id)
        .bind(&link.target_title)
        .fetch_optional(state.pg()?)
        .await
        {
            let _ = sqlx::query("UPDATE wiki_links SET resolved_node_id = $1 WHERE id = $2")
                .bind(resolved.id)
                .bind(link.id)
                .execute(state.pg()?)
                .await;

            let _ = state.neo4j()?.run(
                neo4rs::query("MATCH (a:Node {pg_id: $a_id}), (b:Node {pg_id: $b_id}) CREATE (a)-[:LINKS_TO {context: 'obsidian-import'}]->(b)")
                    .param("a_id", link.source_node_id.to_string())
                    .param("b_id", resolved.id.to_string()),
            ).await;
        }
    }

    Ok(ObsidianImportResult { nodes_created, links_created, files_scanned })
}
