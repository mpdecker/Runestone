use crate::error::AppResult;
use crate::models::node::{Node, NodeListItem};
use crate::repositories::node_repo;
use crate::services::graph_sync;
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

pub fn content_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpsertAction {
    Created,
    Updated,
    Skipped,
}

pub async fn upsert_from_file(
    pool: &PgPool,
    graph: &Arc<neo4rs::Graph>,
    vault_id: Uuid,
    file_path: &str,
    title: &str,
    content: &str,
) -> AppResult<(Node, UpsertAction)> {
    let wc = content.split_whitespace().count() as i32;
    let hash = content_hash(content);

    if let Some(existing) = node_repo::find_by_file_path(pool, vault_id, file_path).await? {
        let node = node_repo::get_by_id(pool, existing.id).await?;
        let existing_hash = content_hash(&node.content);
        if existing_hash == hash {
            return Ok((node, UpsertAction::Skipped));
        }

        let updated = sqlx::query_as::<_, Node>(
            "UPDATE nodes SET title = $2, content = $3, word_count = $4, updated_at = NOW() WHERE id = $1 RETURNING id, vault_id, title, content, content_type, file_path, metadata, word_count, created_at, updated_at",
        )
        .bind(existing.id)
        .bind(title)
        .bind(content)
        .bind(wc)
        .fetch_one(pool)
        .await?;

        graph_sync::update_node(graph, existing.id, &updated.title, &updated.content_type).await?;
        return Ok((updated, UpsertAction::Updated));
    }

    let id = Uuid::new_v4();
    let node = sqlx::query_as::<_, Node>(
        "INSERT INTO nodes (id, vault_id, title, content, content_type, file_path, word_count) VALUES ($1, $2, $3, $4, 'note', $5, $6) RETURNING id, vault_id, title, content, content_type, file_path, metadata, word_count, created_at, updated_at",
    )
    .bind(id)
    .bind(vault_id)
    .bind(title)
    .bind(content)
    .bind(file_path)
    .bind(wc)
    .fetch_one(pool)
    .await?;

    graph_sync::create_node_with_pg_rollback(graph, pool, id, vault_id, &node.title, &node.content_type)
        .await?;

    Ok((node, UpsertAction::Created))
}

pub async fn delete_by_path(
    pool: &PgPool,
    graph: &Arc<neo4rs::Graph>,
    vault_id: Uuid,
    file_path: &str,
) -> AppResult<bool> {
    if let Some(existing) = node_repo::find_by_file_path(pool, vault_id, file_path).await? {
        graph_sync::delete_node_before_pg(graph, existing.id).await?;
        node_repo::delete_by_id(pool, existing.id).await?;
        return Ok(true);
    }
    Ok(false)
}

pub async fn sync_file(
    pool: &PgPool,
    graph: &Arc<neo4rs::Graph>,
    vault_id: Uuid,
    file_path: &str,
    content: &str,
) -> AppResult<SyncFileResult> {
    let title = std::path::Path::new(file_path)
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let (node, action) = upsert_from_file(pool, graph, vault_id, file_path, &title, content).await?;

    Ok(SyncFileResult {
        node: NodeListItem {
            id: node.id,
            title: node.title,
            content_type: node.content_type,
            file_path: Some(file_path.to_string()),
            updated_at: node.updated_at,
        },
        action,
    })
}

#[derive(Debug, Clone)]
pub struct SyncFileResult {
    pub node: NodeListItem,
    pub action: UpsertAction,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_hash_is_deterministic() {
        assert_eq!(content_hash("hello"), content_hash("hello"));
        assert_ne!(content_hash("hello"), content_hash("world"));
    }
}
