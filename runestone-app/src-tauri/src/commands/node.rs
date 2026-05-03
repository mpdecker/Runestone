use crate::embedding::generate_embedding;
use crate::models::node::{CreateNodeRequest, Node, NodeIdRow, NodeListItem, UpdateNodeRequest};
use crate::models::vault::Vault;
use crate::state::AppState;
use uuid::Uuid;

#[tauri::command]
pub async fn create_node(
    state: tauri::State<'_, AppState>,
    request: CreateNodeRequest,
) -> Result<Node, String> {
    let id = Uuid::new_v4();
    let content_type = request.content_type.unwrap_or_else(|| "note".to_string());

    let row = sqlx::query_as::<_, Node>(
        "INSERT INTO nodes (id, vault_id, title, content, content_type, file_path) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id, vault_id, title, content, content_type, file_path, metadata, word_count, created_at, updated_at",
    )
    .bind(id)
    .bind(request.vault_id)
    .bind(&request.title)
    .bind(&request.content)
    .bind(&content_type)
    .bind(&request.file_path)
    .fetch_one(state.pg()?)
    .await
    .map_err(|e| format!("Failed to insert node into PostgreSQL: {}", e))?;

    let pg_id = id.to_string();
        let neo4j_result = state
        .neo4j()?
        .run(
            neo4rs::query("CREATE (n:Node {pg_id: $pg_id, vault_id: $vault_id, title: $title, content_type: $content_type})")
                .param("pg_id", pg_id)
                .param("vault_id", request.vault_id.to_string())
                .param("title", row.title.clone())
                .param("content_type", row.content_type.clone()),
        )
        .await;

    if let Err(e) = neo4j_result {
        let _ = sqlx::query("DELETE FROM nodes WHERE id = $1")
            .bind(id)
            .execute(state.pg()?)
            .await;
        return Err(format!("Neo4j insert failed, rolled back PostgreSQL: {}", e));
    }

    if !row.content.is_empty() {
        let pg = state.pg()?.clone();
        let config = state.embed_config.clone();
        let node_id = id;
        let embed_text = format!("{}: {}", row.title, row.content);
        tokio::spawn(async move {
            match generate_embedding(&embed_text, &config).await {
                Ok(embedding) => {
                    let vector = pgvector::Vector::from(embedding);
                    let _ = sqlx::query("UPDATE nodes SET embedding = $1 WHERE id = $2")
                        .bind(vector)
                        .bind(node_id)
                        .execute(&pg)
                        .await;
                }
                Err(e) => {
                    log::warn!("Failed to generate embedding for node {}: {}", node_id, e);
                }
            }
        });
    }

    Ok(row)
}

#[tauri::command]
pub async fn update_node(
    state: tauri::State<'_, AppState>,
    request: UpdateNodeRequest,
) -> Result<Node, String> {
    let current = sqlx::query_as::<_, Node>(
        "SELECT id, vault_id, title, content, content_type, file_path, metadata, word_count, created_at, updated_at FROM nodes WHERE id = $1",
    )
    .bind(request.id)
    .fetch_one(state.pg()?)
    .await
    .map_err(|e| format!("Node not found: {}", e))?;

    let new_title = request.title.unwrap_or(current.title.clone());
    let new_content = request.content.unwrap_or(current.content.clone());
    let new_content_type = request.content_type.unwrap_or(current.content_type.clone());
    let word_count = new_content.split_whitespace().count() as i32;

    let changed = current.content != new_content || current.title != new_title;
    if changed {
        let version_num = sqlx::query_as::<_, (Option<i32>,)>(
            "SELECT COALESCE(MAX(version_number), 0) + 1 FROM node_versions WHERE node_id = $1",
        )
        .bind(request.id)
        .fetch_one(state.pg()?)
        .await
        .map_err(|e| format!("Version query failed: {}", e))?
        .0
        .unwrap_or(1);

        sqlx::query(
            "INSERT INTO node_versions (node_id, version_number, title, content, word_count) VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(request.id)
        .bind(version_num)
        .bind(&current.title)
        .bind(&current.content)
        .bind(current.word_count)
        .execute(state.pg()?)
        .await
        .map_err(|e| format!("Failed to save version: {}", e))?;
    }

    let row = sqlx::query_as::<_, Node>(
        "UPDATE nodes SET title = $2, content = $3, content_type = $4, word_count = $5, updated_at = NOW() WHERE id = $1 RETURNING id, vault_id, title, content, content_type, file_path, metadata, word_count, created_at, updated_at",
    )
    .bind(request.id)
    .bind(&new_title)
    .bind(&new_content)
    .bind(&new_content_type)
    .bind(word_count)
    .fetch_one(state.pg()?)
    .await
    .map_err(|e| format!("Failed to update node: {}", e))?;

    let _ = state
        .neo4j()?
        .run(
            neo4rs::query("MATCH (n:Node {pg_id: $pg_id}) SET n.title = $title, n.content_type = $content_type")
                .param("pg_id", request.id.to_string())
                .param("title", row.title.clone())
                .param("content_type", row.content_type.clone()),
        )
        .await;

    let pg = state.pg()?.clone();
    let config = state.embed_config.clone();
    let node_id = request.id;
    let embed_text = format!("{}: {}", row.title, row.content);
    tokio::spawn(async move {
        match generate_embedding(&embed_text, &config).await {
            Ok(embedding) => {
                let vector = pgvector::Vector::from(embedding);
                let _ = sqlx::query("UPDATE nodes SET embedding = $1 WHERE id = $2")
                    .bind(vector)
                    .bind(node_id)
                    .execute(&pg)
                    .await;
            }
            Err(e) => {
                log::warn!("Failed to regenerate embedding for node {}: {}", node_id, e);
            }
        }
    });

    Ok(row)
}

#[tauri::command]
pub async fn delete_node(
    state: tauri::State<'_, AppState>,
    id: Uuid,
) -> Result<(), String> {
    let _ = state
        .neo4j()?
        .run(
            neo4rs::query("MATCH (n:Node {pg_id: $pg_id}) DETACH DELETE n")
                .param("pg_id", id.to_string()),
        )
        .await;

    sqlx::query("DELETE FROM nodes WHERE id = $1")
        .bind(id)
        .execute(state.pg()?)
        .await
        .map_err(|e| format!("Failed to delete PostgreSQL node: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn get_node(
    state: tauri::State<'_, AppState>,
    id: Uuid,
) -> Result<Node, String> {
    let row = sqlx::query_as::<_, Node>(
        "SELECT id, vault_id, title, content, content_type, file_path, metadata, word_count, created_at, updated_at FROM nodes WHERE id = $1",
    )
    .bind(id)
    .fetch_one(state.pg()?)
    .await
    .map_err(|e| format!("Node not found: {}", e))?;

    Ok(row)
}

#[tauri::command]
pub async fn list_nodes(
    state: tauri::State<'_, AppState>,
    vault_id: Uuid,
) -> Result<Vec<NodeListItem>, String> {
    let nodes = sqlx::query_as::<_, NodeListItem>(
        "SELECT id, title, content_type, file_path, updated_at FROM nodes WHERE vault_id = $1 ORDER BY updated_at DESC",
    )
    .bind(vault_id)
    .fetch_all(state.pg()?)
    .await
    .map_err(|e| format!("Failed to list nodes: {}", e))?;

    Ok(nodes)
}

#[tauri::command]
pub async fn scan_vault(
    state: tauri::State<'_, AppState>,
    vault_id: Uuid,
) -> Result<Vec<NodeListItem>, String> {
    let vault = sqlx::query_as::<_, Vault>(
        "SELECT id, name, root_path, created_at, updated_at FROM vaults WHERE id = $1",
    )
    .bind(vault_id)
    .fetch_one(state.pg()?)
    .await
    .map_err(|e| format!("Vault not found: {}", e))?;

    let mut created_nodes: Vec<NodeListItem> = Vec::new();

    for entry in walkdir::WalkDir::new(&vault.root_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "md"))
    {
        let file_path = entry.path().to_string_lossy().to_string();
        let title = entry
            .path()
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let existing = sqlx::query_as::<_, NodeIdRow>(
            "SELECT id FROM nodes WHERE vault_id = $1 AND file_path = $2",
        )
        .bind(vault_id)
        .bind(&file_path)
        .fetch_optional(state.pg()?)
        .await
        .map_err(|e| format!("Query error: {}", e))?;

        if existing.is_some() {
            continue;
        }

        let content = std::fs::read_to_string(entry.path()).unwrap_or_default();
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
        .map_err(|e| format!("Failed to insert node: {}", e))?;

        let _ = state
            .neo4j()?
            .run(
                neo4rs::query("CREATE (n:Node {pg_id: $pg_id, vault_id: $vault_id, title: $title, content_type: 'note'})")
                    .param("pg_id", id.to_string())
                    .param("vault_id", vault_id.to_string())
                    .param("title", node.title.clone()),
            )
            .await;

        created_nodes.push(NodeListItem {
            id: node.id,
            title: node.title,
            content_type: node.content_type,
            file_path: Some(file_path.clone()),
            updated_at: node.updated_at,
        });
    }

    Ok(created_nodes)
}

#[tauri::command]
pub async fn get_random_node(
    state: tauri::State<'_, AppState>,
    vault_id: Uuid,
) -> Result<Node, String> {
    let node = sqlx::query_as::<_, Node>(
        "SELECT id, vault_id, title, content, content_type, file_path, metadata, word_count, created_at, updated_at FROM nodes WHERE vault_id = $1 ORDER BY RANDOM() LIMIT 1",
    )
    .bind(vault_id)
    .fetch_one(state.pg()?)
    .await
    .map_err(|e| format!("No nodes found: {}", e))?;

    Ok(node)
}
