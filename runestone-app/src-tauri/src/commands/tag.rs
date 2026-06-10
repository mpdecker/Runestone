use crate::router::dispatch;
use crate::models::node::NodeListItem;
use crate::models::tag::{AddTagsRequest, RemoveTagRequest, TagInfo, TagsResponse};
use crate::state::AppState;
use uuid::Uuid;

#[tauri::command]
pub async fn get_node_tags(
    state: tauri::State<'_, AppState>,
    node_id: Uuid,
) -> Result<TagsResponse, String> {
    dispatch(
        &state,
        "get_node_tags",
        serde_json::to_value(node_id).map_err(|e| e.to_string())?,
    )
    .await
}

#[tauri::command]
pub async fn add_tags_to_node(
    state: tauri::State<'_, AppState>,
    request: AddTagsRequest,
) -> Result<TagsResponse, String> {
    dispatch(
        &state,
        "add_tags_to_node",
        serde_json::to_value(request).map_err(|e| e.to_string())?,
    )
    .await
}

#[tauri::command]
pub async fn remove_tag_from_node(
    state: tauri::State<'_, AppState>,
    request: RemoveTagRequest,
) -> Result<TagsResponse, String> {
    dispatch(
        &state,
        "remove_tag_from_node",
        serde_json::to_value(request).map_err(|e| e.to_string())?,
    )
    .await
}

#[tauri::command]
pub async fn list_tags(
    state: tauri::State<'_, AppState>,
    vault_id: Uuid,
) -> Result<Vec<TagInfo>, String> {
    dispatch(
        &state,
        "list_tags",
        serde_json::to_value(vault_id).map_err(|e| e.to_string())?,
    )
    .await
}

#[tauri::command]
pub async fn get_nodes_by_tag(
    state: tauri::State<'_, AppState>,
    vault_id: Uuid,
    tag: String,
) -> Result<Vec<NodeListItem>, String> {
    dispatch(
        &state,
        "get_nodes_by_tag",
        serde_json::json!({ "vault_id": vault_id, "tag": tag }),
    )
    .await
}

#[tauri::command]
pub async fn accept_tag_suggestions(
    state: tauri::State<'_, AppState>,
    node_id: Uuid,
    tags: Vec<String>,
) -> Result<TagsResponse, String> {
    dispatch(
        &state,
        "accept_tag_suggestions",
        serde_json::json!({ "node_id": node_id, "tags": tags }),
    )
    .await
}
