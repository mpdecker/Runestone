use crate::router::dispatch;
use crate::models::node::{Node, NodeListItem};
use crate::state::AppState;
use uuid::Uuid;

#[tauri::command]
pub async fn create_daily_note(
    state: tauri::State<'_, AppState>,
    vault_id: Uuid,
) -> Result<Node, String> {
    dispatch(
        &state,
        "create_daily_note",
        serde_json::to_value(vault_id).map_err(|e| e.to_string())?,
    )
    .await
}

#[tauri::command]
pub async fn list_templates(
    state: tauri::State<'_, AppState>,
    vault_id: Uuid,
) -> Result<Vec<NodeListItem>, String> {
    dispatch(
        &state,
        "list_templates",
        serde_json::to_value(vault_id).map_err(|e| e.to_string())?,
    )
    .await
}

#[tauri::command]
pub async fn create_node_from_template(
    state: tauri::State<'_, AppState>,
    template_id: Uuid,
    title: Option<String>,
) -> Result<Node, String> {
    dispatch(
        &state,
        "create_node_from_template",
        serde_json::json!({ "template_id": template_id, "title": title }),
    )
    .await
}
