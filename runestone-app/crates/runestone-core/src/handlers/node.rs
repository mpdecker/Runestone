use crate::context::BackendContext;
use crate::handlers::embeddings;
use crate::models::node::{CreateNodeRequest, ListNodesRequest, Node, NodeListItem, UpdateNodeRequest};
use crate::repositories::node_repo;
use crate::services::graph_sync;
use uuid::Uuid;

pub async fn create_node(
    ctx: &BackendContext,
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
    .fetch_one(&ctx.pg)
    .await
    .map_err(|e| format!("Failed to insert node into PostgreSQL: {}", e))?;

    graph_sync::create_node_with_pg_rollback(
        &ctx.neo4j,
        &ctx.pg,
        id,
        request.vault_id,
        &row.title,
        &row.content_type,
    )
    .await
    .map_err(|e| e.to_string())?;

    if !row.content.is_empty() {
        embeddings::enqueue_embedding(ctx, id).await?;
    }

    Ok(row)
}

pub async fn update_node(
    ctx: &BackendContext,
    request: UpdateNodeRequest,
) -> Result<Node, String> {
    let current = node_repo::get_by_id(&ctx.pg, request.id)
        .await
        .map_err(|e| e.to_string())?;

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
        .fetch_one(&ctx.pg)
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
        .execute(&ctx.pg)
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
    .fetch_one(&ctx.pg)
    .await
    .map_err(|e| format!("Failed to update node: {}", e))?;

    graph_sync::update_node(
        &ctx.neo4j,
        request.id,
        &row.title,
        &row.content_type,
    )
    .await
    .map_err(|e| format!("Neo4j update failed: {}", e))?;

    embeddings::enqueue_embedding(ctx, request.id).await?;

    Ok(row)
}

pub async fn delete_node(ctx: &BackendContext, id: Uuid) -> Result<(), String> {
    graph_sync::delete_node_before_pg(&ctx.neo4j, id)
        .await
        .map_err(|e| format!("Neo4j delete failed: {}", e))?;

    node_repo::delete_by_id(&ctx.pg, id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

pub async fn get_node(ctx: &BackendContext, id: Uuid) -> Result<Node, String> {
    node_repo::get_by_id(&ctx.pg, id)
        .await
        .map_err(|e| e.to_string())
}

pub async fn list_nodes(
    ctx: &BackendContext,
    request: ListNodesRequest,
) -> Result<Vec<NodeListItem>, String> {
    if let (Some(limit), Some(offset)) = (request.limit, request.offset) {
        return node_repo::list_nodes_paginated(&ctx.pg, request.vault_id, limit, offset)
            .await
            .map_err(|e| e.to_string());
    }
    node_repo::list_by_vault(&ctx.pg, request.vault_id)
        .await
        .map_err(|e| e.to_string())
}

pub async fn get_random_node(ctx: &BackendContext, vault_id: Uuid) -> Result<Node, String> {
    let node = sqlx::query_as::<_, Node>(
        "SELECT id, vault_id, title, content, content_type, file_path, metadata, word_count, created_at, updated_at FROM nodes WHERE vault_id = $1 ORDER BY RANDOM() LIMIT 1",
    )
    .bind(vault_id)
    .fetch_one(&ctx.pg)
    .await
    .map_err(|e| format!("No nodes found: {}", e))?;

    Ok(node)
}
