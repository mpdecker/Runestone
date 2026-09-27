use crate::context::BackendContext;
use crate::error::AppError;
use crate::handlers::{embeddings, node as node_handler};
use crate::models::node::Node;
use crate::models::version::NodeVersion;
use crate::services::graph_sync;
use uuid::Uuid;

pub async fn get_node_versions(
    ctx: &BackendContext,
    node_id: Uuid,
) -> Result<Vec<NodeVersion>, String> {
    let versions = sqlx::query_as::<_, NodeVersion>(
        "SELECT id, node_id, version_number, title, content, word_count, created_at FROM node_versions WHERE node_id = $1 ORDER BY version_number DESC LIMIT 50",
    )
    .bind(node_id)
    .fetch_all(&ctx.pg)
    .await
    .map_err(|e| format!("Failed to load versions: {}", e))?;

    Ok(versions)
}

/// Restore a snapshot. The content being replaced is itself snapshotted first, so a restore can
/// always be undone (previously the newest edits were simply overwritten and unrecoverable).
pub async fn restore_node_version(ctx: &BackendContext, version_id: Uuid) -> Result<Node, String> {
    let version = sqlx::query_as::<_, NodeVersion>(
        "SELECT id, node_id, version_number, title, content, word_count, created_at FROM node_versions WHERE id = $1",
    )
    .bind(version_id)
    .fetch_optional(&ctx.pg)
    .await
    .map_err(|e| format!("Failed to load version: {}", e))?
    .ok_or_else(|| AppError::NotFound(format!("Version {} not found", version_id)).to_string())?;

    let mut tx = ctx
        .pg
        .begin()
        .await
        .map_err(|e| format!("Failed to start transaction: {}", e))?;

    let current = sqlx::query_as::<_, Node>(
        "SELECT id, vault_id, title, content, content_type, file_path, metadata, word_count, created_at, updated_at FROM nodes WHERE id = $1 FOR UPDATE",
    )
    .bind(version.node_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| format!("Failed to load node: {}", e))?
    .ok_or_else(|| AppError::NotFound(format!("Node {} not found", version.node_id)).to_string())?;

    let title_changed = current.title != version.title;
    if current.content != version.content || title_changed {
        node_handler::insert_version(&mut tx, &current).await?;
    }

    let node = sqlx::query_as::<_, Node>(
        "UPDATE nodes SET title = $2, content = $3, word_count = $4, updated_at = NOW() WHERE id = $1 RETURNING id, vault_id, title, content, content_type, file_path, metadata, word_count, created_at, updated_at",
    )
    .bind(version.node_id)
    .bind(&version.title)
    .bind(&version.content)
    .bind(version.word_count)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| format!("Failed to restore version: {}", e))?;

    tx.commit()
        .await
        .map_err(|e| format!("Failed to commit restore: {}", e))?;

    graph_sync::update_node(&ctx.neo4j, node.id, &node.title, &node.content_type)
        .await
        .map_err(|e| e.to_string())?;

    node_handler::sync_links_best_effort(ctx, &node, title_changed).await;
    embeddings::enqueue_embedding(ctx, node.id).await?;

    Ok(node)
}
