use crate::router::dispatch;
use crate::models::node::Node;
use runestone_core::models::version::NodeVersion;
use crate::state::AppState;
use uuid::Uuid;

#[tauri::command]
pub async fn get_node_versions(
    state: tauri::State<'_, AppState>,
    node_id: Uuid,
) -> Result<Vec<NodeVersion>, String> {
    dispatch(
        &state,
        "get_node_versions",
        serde_json::to_value(node_id).map_err(|e| e.to_string())?,
    )
    .await
}

#[tauri::command]
pub async fn restore_node_version(
    state: tauri::State<'_, AppState>,
    version_id: Uuid,
) -> Result<Node, String> {
    dispatch(
        &state,
        "restore_node_version",
        serde_json::to_value(version_id).map_err(|e| e.to_string())?,
    )
    .await
}
