use crate::router::dispatch;
use crate::models::graph::{Backlink, GraphData, GraphOptions};
use crate::models::graph::WikiLinkRow;
use crate::state::AppState;
use uuid::Uuid;

#[tauri::command]
pub async fn get_graph_data(
    state: tauri::State<'_, AppState>,
    vault_id: Uuid,
    options: Option<GraphOptions>,
) -> Result<GraphData, String> {
    dispatch(
        &state,
        "get_graph_data",
        serde_json::json!({ "vault_id": vault_id, "options": options }),
    )
    .await
}

#[tauri::command]
pub async fn get_local_graph(
    state: tauri::State<'_, AppState>,
    node_id: Uuid,
    depth: Option<u32>,
) -> Result<GraphData, String> {
    dispatch(
        &state,
        "get_local_graph",
        serde_json::json!({ "node_id": node_id, "depth": depth }),
    )
    .await
}

#[tauri::command]
pub async fn parse_wiki_links(
    state: tauri::State<'_, AppState>,
    node_id: Uuid,
) -> Result<Vec<WikiLinkRow>, String> {
    dispatch(
        &state,
        "parse_wiki_links",
        serde_json::to_value(node_id).map_err(|e| e.to_string())?,
    )
    .await
}

#[tauri::command]
pub async fn get_backlinks(
    state: tauri::State<'_, AppState>,
    node_id: Uuid,
) -> Result<Vec<Backlink>, String> {
    dispatch(
        &state,
        "get_backlinks",
        serde_json::to_value(node_id).map_err(|e| e.to_string())?,
    )
    .await
}

#[tauri::command]
pub async fn get_outgoing_links(
    state: tauri::State<'_, AppState>,
    node_id: Uuid,
) -> Result<Vec<Backlink>, String> {
    dispatch(
        &state,
        "get_outgoing_links",
        serde_json::to_value(node_id).map_err(|e| e.to_string())?,
    )
    .await
}
