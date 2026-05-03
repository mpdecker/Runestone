use crate::models::node::Node;
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct NodeVersion {
    pub id: Uuid,
    pub node_id: Uuid,
    pub version_number: i32,
    pub title: String,
    pub content: String,
    pub word_count: i32,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[tauri::command]
pub async fn get_node_versions(
    state: tauri::State<'_, AppState>,
    node_id: Uuid,
) -> Result<Vec<NodeVersion>, String> {
    let versions = sqlx::query_as::<_, NodeVersion>(
        "SELECT id, node_id, version_number, title, content, word_count, created_at FROM node_versions WHERE node_id = $1 ORDER BY version_number DESC LIMIT 50",
    )
    .bind(node_id)
    .fetch_all(state.pg()?)
    .await
    .map_err(|e| format!("Failed to load versions: {}", e))?;

    Ok(versions)
}

#[tauri::command]
pub async fn restore_node_version(
    state: tauri::State<'_, AppState>,
    version_id: Uuid,
) -> Result<Node, String> {
    let version = sqlx::query_as::<_, NodeVersion>(
        "SELECT id, node_id, version_number, title, content, word_count, created_at FROM node_versions WHERE id = $1",
    )
    .bind(version_id)
    .fetch_one(state.pg()?)
    .await
    .map_err(|e| format!("Version not found: {}", e))?;

    let node = sqlx::query_as::<_, Node>(
        "UPDATE nodes SET title = $2, content = $3, word_count = $4, updated_at = NOW() WHERE id = $1 RETURNING id, vault_id, title, content, content_type, file_path, metadata, word_count, created_at, updated_at",
    )
    .bind(version.node_id)
    .bind(&version.title)
    .bind(&version.content)
    .bind(version.word_count)
    .fetch_one(state.pg()?)
    .await
    .map_err(|e| format!("Failed to restore version: {}", e))?;

    Ok(node)
}
