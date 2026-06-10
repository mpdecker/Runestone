use crate::router::dispatch;
use crate::models::node::Node;
use crate::models::search::{SearchQuery, SearchResult, SearchResults};
use crate::state::AppState;
use uuid::Uuid;

#[tauri::command]
pub async fn semantic_search(
    state: tauri::State<'_, AppState>,
    query: SearchQuery,
) -> Result<Vec<SearchResult>, String> {
    dispatch(
        &state,
        "semantic_search",
        serde_json::to_value(query).map_err(|e| e.to_string())?,
    )
    .await
}

#[tauri::command]
pub async fn find_similar(
    state: tauri::State<'_, AppState>,
    node_id: Uuid,
    limit: Option<i64>,
) -> Result<Vec<SearchResult>, String> {
    dispatch(
        &state,
        "find_similar",
        serde_json::json!({ "node_id": node_id, "limit": limit }),
    )
    .await
}

#[tauri::command]
pub async fn hybrid_search(
    state: tauri::State<'_, AppState>,
    query: SearchQuery,
) -> Result<SearchResults, String> {
    dispatch(
        &state,
        "hybrid_search",
        serde_json::to_value(query).map_err(|e| e.to_string())?,
    )
    .await
}

#[tauri::command]
pub async fn boolean_search(
    state: tauri::State<'_, AppState>,
    query: SearchQuery,
) -> Result<Vec<SearchResult>, String> {
    dispatch(
        &state,
        "boolean_search",
        serde_json::to_value(query).map_err(|e| e.to_string())?,
    )
    .await
}

#[tauri::command]
pub async fn regex_search(
    state: tauri::State<'_, AppState>,
    vault_id: Uuid,
    pattern: String,
    case_sensitive: Option<bool>,
    limit: Option<i64>,
) -> Result<Vec<SearchResult>, String> {
    dispatch(
        &state,
        "regex_search",
        serde_json::json!({
            "vault_id": vault_id,
            "pattern": pattern,
            "case_sensitive": case_sensitive,
            "limit": limit,
        }),
    )
    .await
}

#[tauri::command]
pub async fn get_node_by_alias(
    state: tauri::State<'_, AppState>,
    vault_id: Uuid,
    alias: String,
) -> Result<Option<Node>, String> {
    dispatch(
        &state,
        "get_node_by_alias",
        serde_json::json!({ "vault_id": vault_id, "alias": alias }),
    )
    .await
}

#[tauri::command]
pub async fn add_alias(
    state: tauri::State<'_, AppState>,
    node_id: Uuid,
    alias: String,
) -> Result<serde_json::Value, String> {
    dispatch(
        &state,
        "add_alias",
        serde_json::json!({ "node_id": node_id, "alias": alias }),
    )
    .await
}

#[tauri::command]
pub async fn remove_alias(
    state: tauri::State<'_, AppState>,
    node_id: Uuid,
    alias: String,
) -> Result<serde_json::Value, String> {
    dispatch(
        &state,
        "remove_alias",
        serde_json::json!({ "node_id": node_id, "alias": alias }),
    )
    .await
}
