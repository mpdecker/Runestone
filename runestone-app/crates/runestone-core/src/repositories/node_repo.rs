use crate::error::{AppError, AppResult};
use crate::models::node::{Node, NodeIdRow, NodeListItem};
use sqlx::PgPool;
use uuid::Uuid;

pub const NODE_SELECT: &str =
    "SELECT id, vault_id, title, content, content_type, file_path, metadata, word_count, created_at, updated_at FROM nodes";

pub async fn get_by_id(pool: &PgPool, id: Uuid) -> AppResult<Node> {
    sqlx::query_as::<_, Node>(
        "SELECT id, vault_id, title, content, content_type, file_path, metadata, word_count, created_at, updated_at FROM nodes WHERE id = $1",
    )
        .bind(id)
        .fetch_one(pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => AppError::NotFound(format!("Node {} not found", id)),
            other => AppError::Postgres(other.to_string()),
        })
}

pub async fn list_by_vault(pool: &PgPool, vault_id: Uuid) -> AppResult<Vec<NodeListItem>> {
    Ok(sqlx::query_as::<_, NodeListItem>(
        "SELECT id, title, content_type, file_path, updated_at FROM nodes WHERE vault_id = $1 ORDER BY updated_at DESC",
    )
    .bind(vault_id)
    .fetch_all(pool)
    .await?)
}

pub async fn list_nodes_paginated(
    pool: &PgPool,
    vault_id: Uuid,
    limit: i64,
    offset: i64,
) -> AppResult<Vec<NodeListItem>> {
    Ok(sqlx::query_as::<_, NodeListItem>(
        "SELECT id, title, content_type, file_path, updated_at FROM nodes WHERE vault_id = $1 ORDER BY updated_at DESC LIMIT $2 OFFSET $3",
    )
    .bind(vault_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?)
}

pub async fn find_by_file_path(
    pool: &PgPool,
    vault_id: Uuid,
    file_path: &str,
) -> AppResult<Option<NodeIdRow>> {
    Ok(sqlx::query_as::<_, NodeIdRow>(
        "SELECT id FROM nodes WHERE vault_id = $1 AND file_path = $2",
    )
    .bind(vault_id)
    .bind(file_path)
    .fetch_optional(pool)
    .await?)
}

pub async fn find_by_title(
    pool: &PgPool,
    vault_id: Uuid,
    title: &str,
) -> AppResult<Option<NodeIdRow>> {
    Ok(sqlx::query_as::<_, NodeIdRow>(
        "SELECT id FROM nodes WHERE vault_id = $1 AND title = $2 LIMIT 1",
    )
    .bind(vault_id)
    .bind(title)
    .fetch_optional(pool)
    .await?)
}

pub async fn delete_by_id(pool: &PgPool, id: Uuid) -> AppResult<()> {
    sqlx::query("DELETE FROM nodes WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}
