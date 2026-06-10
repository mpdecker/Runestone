use crate::router::dispatch;
use crate::models::node::Node;
use crate::state::AppState;
use uuid::Uuid;

#[tauri::command]
pub async fn merge_nodes(
    state: tauri::State<'_, AppState>,
    source_id: Uuid,
    target_id: Uuid,
) -> Result<Node, String> {
    dispatch(
        &state,
        "merge_nodes",
        serde_json::json!({ "source_id": source_id, "target_id": target_id }),
    )
    .await
}

#[tauri::command]
pub async fn split_node(
    state: tauri::State<'_, AppState>,
    node_id: Uuid,
    new_title: String,
) -> Result<(Node, Node), String> {
    dispatch(
        &state,
        "split_node",
        serde_json::json!({ "node_id": node_id, "new_title": new_title }),
    )
    .await
}
