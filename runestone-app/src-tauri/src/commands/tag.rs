use crate::models::tag::{AddTagsRequest, RemoveTagRequest, TagInfo, TagsResponse};
use crate::models::node::NodeListItem;
use crate::state::AppState;
use uuid::Uuid;

#[tauri::command]
pub async fn get_node_tags(
    state: tauri::State<'_, AppState>,
    node_id: Uuid,
) -> Result<TagsResponse, String> {
    let row = sqlx::query_as::<_, (Option<serde_json::Value>,)>(
        "SELECT metadata FROM nodes WHERE id = $1",
    )
    .bind(node_id)
    .fetch_one(state.pg()?)
    .await
    .map_err(|e| format!("Node not found: {}", e))?;

    let tags = row
        .0
        .and_then(|v| v.get("tags").cloned())
        .and_then(|v| serde_json::from_value::<Vec<String>>(v).ok())
        .unwrap_or_default();

    Ok(TagsResponse {
        node_id,
        tags,
    })
}

#[tauri::command]
pub async fn add_tags_to_node(
    state: tauri::State<'_, AppState>,
    request: AddTagsRequest,
) -> Result<TagsResponse, String> {
    let current_meta = sqlx::query_as::<_, (Option<serde_json::Value>,)>(
        "SELECT metadata FROM nodes WHERE id = $1",
    )
    .bind(request.node_id)
    .fetch_one(state.pg()?)
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
        .execute(state.pg()?)
        .await
        .map_err(|e| format!("Failed to update tags: {}", e))?;

    for tag in &new_tags {
        if !existing_tags.contains(tag) {
            let _ = state
                .neo4j()?
                .run(
                    neo4rs::query(
                        "MERGE (t:Tag {name: $tag_name}) WITH t MATCH (n:Node {pg_id: $pg_id}) MERGE (n)-[:HAS_TAG]->(t)",
                    )
                    .param("tag_name", tag.as_str())
                    .param("pg_id", request.node_id.to_string()),
                )
                .await;
        }
    }

    Ok(TagsResponse {
        node_id: request.node_id,
        tags: new_tags,
    })
}

#[tauri::command]
pub async fn remove_tag_from_node(
    state: tauri::State<'_, AppState>,
    request: RemoveTagRequest,
) -> Result<TagsResponse, String> {
    let current_meta = sqlx::query_as::<_, (Option<serde_json::Value>,)>(
        "SELECT metadata FROM nodes WHERE id = $1",
    )
    .bind(request.node_id)
    .fetch_one(state.pg()?)
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
        .execute(state.pg()?)
        .await
        .map_err(|e| format!("Failed to update tags: {}", e))?;

    let _ = state
        .neo4j()?
        .run(
            neo4rs::query(
                "MATCH (n:Node {pg_id: $pg_id})-[r:HAS_TAG]->(t:Tag {name: $tag_name}) DELETE r",
            )
            .param("pg_id", request.node_id.to_string())
            .param("tag_name", request.tag.as_str()),
        )
        .await;

    Ok(TagsResponse {
        node_id: request.node_id,
        tags: new_tags,
    })
}

#[tauri::command]
pub async fn list_tags(
    state: tauri::State<'_, AppState>,
    vault_id: Uuid,
) -> Result<Vec<TagInfo>, String> {
    let rows = sqlx::query_as::<_, (String, i64)>(
        r#"SELECT tag, COUNT(*) as node_count
           FROM nodes, jsonb_array_elements_text(COALESCE(metadata->'tags', '[]'::jsonb)) AS tag
           WHERE vault_id = $1
           GROUP BY tag
           ORDER BY node_count DESC"#,
    )
    .bind(vault_id)
    .fetch_all(state.pg()?)
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

#[tauri::command]
pub async fn get_nodes_by_tag(
    state: tauri::State<'_, AppState>,
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
    .fetch_all(state.pg()?)
    .await
    .map_err(|e| format!("Failed to get nodes by tag: {}", e))?;

    Ok(rows)
}

#[tauri::command]
pub async fn accept_tag_suggestions(
    state: tauri::State<'_, AppState>,
    node_id: Uuid,
    tags: Vec<String>,
) -> Result<TagsResponse, String> {
    add_tags_to_node(
        state,
        AddTagsRequest { node_id, tags },
    )
    .await
}
