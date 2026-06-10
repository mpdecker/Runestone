use crate::context::BackendContext;
use crate::models::node::NodeListItem;
use crate::models::tag::{AddTagsRequest, RemoveTagRequest, TagInfo, TagsResponse};
use crate::services::graph_sync;
use uuid::Uuid;

pub async fn get_node_tags(ctx: &BackendContext, node_id: Uuid) -> Result<TagsResponse, String> {
    let row = sqlx::query_as::<_, (Option<serde_json::Value>,)>(
        "SELECT metadata FROM nodes WHERE id = $1",
    )
    .bind(node_id)
    .fetch_one(&ctx.pg)
    .await
    .map_err(|e| format!("Node not found: {}", e))?;

    let tags = row
        .0
        .and_then(|v| v.get("tags").cloned())
        .and_then(|v| serde_json::from_value::<Vec<String>>(v).ok())
        .unwrap_or_default();

    Ok(TagsResponse { node_id, tags })
}

pub async fn add_tags_to_node(
    ctx: &BackendContext,
    request: AddTagsRequest,
) -> Result<TagsResponse, String> {
    let current_meta = sqlx::query_as::<_, (Option<serde_json::Value>,)>(
        "SELECT metadata FROM nodes WHERE id = $1",
    )
    .bind(request.node_id)
    .fetch_one(&ctx.pg)
    .await
    .map_err(|e| format!("Node not found: {}", e))?;

    let existing_tags: Vec<String> = current_meta
        .0
        .as_ref()
        .and_then(|v| v.get("tags"))
        .and_then(|v| serde_json::from_value::<Vec<String>>(v.clone()).ok())
        .unwrap_or_default();

    let mut new_tags = existing_tags.clone();
    for tag in &request.tags {
        let normalized = tag.trim().to_lowercase();
        if !normalized.is_empty() && !new_tags.contains(&normalized) {
            new_tags.push(normalized);
        }
    }

    let mut meta = current_meta.0.unwrap_or(serde_json::json!({}));
    meta["tags"] = serde_json::json!(new_tags.clone());

    sqlx::query("UPDATE nodes SET metadata = $2, updated_at = NOW() WHERE id = $1")
        .bind(request.node_id)
        .bind(&meta)
        .execute(&ctx.pg)
        .await
        .map_err(|e| format!("Failed to update tags: {}", e))?;

    for tag in &new_tags {
        if !existing_tags.contains(tag) {
            graph_sync::add_tag(&ctx.neo4j, request.node_id, tag)
                .await
                .map_err(|e| format!("Neo4j tag sync failed: {}", e))?;
        }
    }

    Ok(TagsResponse {
        node_id: request.node_id,
        tags: new_tags,
    })
}

pub async fn remove_tag_from_node(
    ctx: &BackendContext,
    request: RemoveTagRequest,
) -> Result<TagsResponse, String> {
    let current_meta = sqlx::query_as::<_, (Option<serde_json::Value>,)>(
        "SELECT metadata FROM nodes WHERE id = $1",
    )
    .bind(request.node_id)
    .fetch_one(&ctx.pg)
    .await
    .map_err(|e| format!("Node not found: {}", e))?;

    let existing_tags: Vec<String> = current_meta
        .0
        .as_ref()
        .and_then(|v| v.get("tags"))
        .and_then(|v| serde_json::from_value::<Vec<String>>(v.clone()).ok())
        .unwrap_or_default();

    let new_tags: Vec<String> = existing_tags
        .iter()
        .filter(|t| **t != request.tag)
        .cloned()
        .collect();

    let mut meta = current_meta.0.unwrap_or(serde_json::json!({}));
    meta["tags"] = serde_json::json!(new_tags.clone());

    sqlx::query("UPDATE nodes SET metadata = $2, updated_at = NOW() WHERE id = $1")
        .bind(request.node_id)
        .bind(&meta)
        .execute(&ctx.pg)
        .await
        .map_err(|e| format!("Failed to update tags: {}", e))?;

    graph_sync::remove_tag(&ctx.neo4j, request.node_id, &request.tag)
        .await
        .map_err(|e| format!("Neo4j tag removal failed: {}", e))?;

    Ok(TagsResponse {
        node_id: request.node_id,
        tags: new_tags,
    })
}

pub async fn list_tags(ctx: &BackendContext, vault_id: Uuid) -> Result<Vec<TagInfo>, String> {
    let rows = sqlx::query_as::<_, (String, i64)>(
        r#"SELECT tag, COUNT(*) as node_count
           FROM nodes, jsonb_array_elements_text(COALESCE(metadata->'tags', '[]'::jsonb)) AS tag
           WHERE vault_id = $1
           GROUP BY tag
           ORDER BY node_count DESC"#,
    )
    .bind(vault_id)
    .fetch_all(&ctx.pg)
    .await
    .map_err(|e| format!("Failed to list tags: {}", e))?;

    Ok(rows
        .into_iter()
        .map(|(name, count)| TagInfo {
            name,
            node_count: Some(count),
        })
        .collect())
}

pub async fn get_nodes_by_tag(
    ctx: &BackendContext,
    vault_id: Uuid,
    tag: String,
) -> Result<Vec<NodeListItem>, String> {
    let rows = sqlx::query_as::<_, NodeListItem>(
        r#"SELECT id, title, content_type, file_path, updated_at
           FROM nodes
           WHERE vault_id = $1 AND metadata->'tags' ? $2
           ORDER BY updated_at DESC"#,
    )
    .bind(vault_id)
    .bind(&tag)
    .fetch_all(&ctx.pg)
    .await
    .map_err(|e| format!("Failed to get nodes by tag: {}", e))?;

    Ok(rows)
}

pub async fn accept_tag_suggestions(
    ctx: &BackendContext,
    node_id: Uuid,
    tags: Vec<String>,
) -> Result<TagsResponse, String> {
    add_tags_to_node(ctx, AddTagsRequest { node_id, tags }).await
}
