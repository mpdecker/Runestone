use crate::router::dispatch;
use runestone_core::models::properties::{PropertiesResponse, SetPropertyRequest};
use crate::state::AppState;
use uuid::Uuid;

#[tauri::command]
pub async fn get_node_properties(
    state: tauri::State<'_, AppState>,
    node_id: Uuid,
) -> Result<PropertiesResponse, String> {
    dispatch(
        &state,
        "get_node_properties",
        serde_json::to_value(node_id).map_err(|e| e.to_string())?,
    )
    .await
}

#[tauri::command]
pub async fn set_node_property(
    state: tauri::State<'_, AppState>,
    request: SetPropertyRequest,
) -> Result<PropertiesResponse, String> {
    dispatch(
        &state,
        "set_node_property",
        serde_json::to_value(request).map_err(|e| e.to_string())?,
    )
    .await
}

#[tauri::command]
pub async fn remove_node_property(
    state: tauri::State<'_, AppState>,
    node_id: Uuid,
    key: String,
) -> Result<PropertiesResponse, String> {
    dispatch(
        &state,
        "remove_node_property",
        serde_json::json!({ "node_id": node_id, "key": key }),
    )
    .await
}
