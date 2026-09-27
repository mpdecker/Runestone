use crate::context::BackendContext;
use crate::error::AppError;
use crate::handlers::{embeddings, graph};
use crate::models::node::{
    CreateNodeRequest, ListNodesRequest, Node, NodeListItem, UpdateNodeRequest,
};
use crate::repositories::node_repo;
use crate::services::graph_sync;
use crate::util::word_count;
use uuid::Uuid;

// sqlx only accepts compile-time SQL, so the shared column list is a literal-producing macro.
macro_rules! node_columns {
    () => {
        "id, vault_id, title, content, content_type, file_path, metadata, word_count, created_at, updated_at"
    };
}

fn validate_title(title: &str) -> Result<(), String> {
    if title.trim().is_empty() {
        return Err(AppError::Validation("Title must not be empty".to_string()).to_string());
    }
    Ok(())
}

pub async fn create_node(ctx: &BackendContext, request: CreateNodeRequest) -> Result<Node, String> {
    validate_title(&request.title)?;
    let id = Uuid::new_v4();
    let content_type = request.content_type.unwrap_or_else(|| "note".to_string());

    let row = sqlx::query_as::<_, Node>(concat!("INSERT INTO nodes (id, vault_id, title, content, content_type, file_path, word_count) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING ", node_columns!(), ""))
    .bind(id)
    .bind(request.vault_id)
    .bind(&request.title)
    .bind(&request.content)
    .bind(&content_type)
    .bind(&request.file_path)
    .bind(word_count(&request.content))
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

    sync_links_best_effort(ctx, &row, true).await;

    if !row.content.is_empty() {
        embeddings::enqueue_embedding(ctx, id).await?;
    }

    Ok(row)
}

/// Keep `wiki_links` / `LINKS_TO` in step with a note that was just written. Failures are
/// logged, not returned: the note itself is already saved and "Parse Links" can repair the graph.
pub(crate) async fn sync_links_best_effort(ctx: &BackendContext, node: &Node, title_changed: bool) {
    if title_changed {
        if let Err(e) = graph::resolve_pending_links(ctx, node.id, node.vault_id, &node.title).await
        {
            log::warn!("Resolving pending links for {} failed: {}", node.id, e);
        }
    }
    if let Err(e) = graph::parse_wiki_links(ctx, node.id).await {
        log::warn!("Wiki link sync for {} failed: {}", node.id, e);
    }
}

pub async fn update_node(ctx: &BackendContext, request: UpdateNodeRequest) -> Result<Node, String> {
    if let Some(title) = &request.title {
        validate_title(title)?;
    }

    // Read-modify-write under a row lock so concurrent saves cannot hand out the same version
    // number, and the version snapshot always matches the row it replaces.
    let mut tx = ctx
        .pg
        .begin()
        .await
        .map_err(|e| format!("Failed to start transaction: {}", e))?;

    let current = sqlx::query_as::<_, Node>(concat!(
        "SELECT ",
        node_columns!(),
        " FROM nodes WHERE id = $1 FOR UPDATE"
    ))
    .bind(request.id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| format!("Failed to load node: {}", e))?
    .ok_or_else(|| AppError::NotFound(format!("Node {} not found", request.id)).to_string())?;

    let new_title = request.title.unwrap_or(current.title.clone());
    let new_content = request.content.unwrap_or(current.content.clone());
    let new_content_type = request.content_type.unwrap_or(current.content_type.clone());
    let wc = word_count(&new_content);

    let title_changed = current.title != new_title;
    let changed = current.content != new_content || title_changed;
    if changed {
        insert_version(&mut tx, &current).await?;
    }

    let row = sqlx::query_as::<_, Node>(concat!("UPDATE nodes SET title = $2, content = $3, content_type = $4, word_count = $5, updated_at = NOW() WHERE id = $1 RETURNING ", node_columns!(), ""))
    .bind(request.id)
    .bind(&new_title)
    .bind(&new_content)
    .bind(&new_content_type)
    .bind(wc)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| format!("Failed to update node: {}", e))?;

    tx.commit()
        .await
        .map_err(|e| format!("Failed to commit node update: {}", e))?;

    graph_sync::update_node(&ctx.neo4j, request.id, &row.title, &row.content_type)
        .await
        .map_err(|e| format!("Neo4j update failed: {}", e))?;

    if changed {
        sync_links_best_effort(ctx, &row, title_changed).await;
    }

    embeddings::enqueue_embedding(ctx, request.id).await?;

    Ok(row)
}

/// Snapshot `node` as the next version. Callers must hold a lock on the node row.
pub(crate) async fn insert_version(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    node: &Node,
) -> Result<(), String> {
    sqlx::query(
        "INSERT INTO node_versions (node_id, version_number, title, content, word_count) SELECT $1, COALESCE(MAX(version_number), 0) + 1, $2, $3, $4 FROM node_versions WHERE node_id = $1",
    )
    .bind(node.id)
    .bind(&node.title)
    .bind(&node.content)
    .bind(node.word_count)
    .execute(&mut **tx)
    .await
    .map_err(|e| format!("Failed to save version: {}", e))?;
    Ok(())
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
    // A limit on its own (no offset) is a first-page request, not "give me everything".
    if let Some(limit) = request.limit {
        return node_repo::list_nodes_paginated(
            &ctx.pg,
            request.vault_id,
            limit.max(0),
            request.offset.unwrap_or(0).max(0),
        )
        .await
        .map_err(|e| e.to_string());
    }
    node_repo::list_by_vault(&ctx.pg, request.vault_id)
        .await
        .map_err(|e| e.to_string())
}

pub async fn get_random_node(ctx: &BackendContext, vault_id: Uuid) -> Result<Node, String> {
    let node = sqlx::query_as::<_, Node>(concat!(
        "SELECT ",
        node_columns!(),
        " FROM nodes WHERE vault_id = $1 ORDER BY RANDOM() LIMIT 1"
    ))
    .bind(vault_id)
    .fetch_optional(&ctx.pg)
    .await
    .map_err(|e| format!("Failed to pick a random note: {}", e))?
    .ok_or_else(|| "No notes in this vault yet".to_string())?;

    Ok(node)
}
