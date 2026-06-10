use runestone_core::handlers::ai;
use crate::models::chat::{ChatRequest, ChatResponse, TagSuggestion};
use crate::router::dispatch;
use crate::state::AppState;
use tauri::Emitter;
use uuid::Uuid;

#[tauri::command]
pub async fn summarize_node(
    state: tauri::State<'_, AppState>,
    node_id: Uuid,
) -> Result<String, String> {
    dispatch(
        &state,
        "summarize_node",
        serde_json::to_value(node_id).map_err(|e| e.to_string())?,
    )
    .await
}

#[tauri::command]
pub async fn chat_with_graph(
    state: tauri::State<'_, AppState>,
    request: ChatRequest,
) -> Result<ChatResponse, String> {
    dispatch(
        &state,
        "chat_with_graph",
        serde_json::to_value(request).map_err(|e| e.to_string())?,
    )
    .await
}

#[tauri::command]
pub async fn chat_with_graph_stream(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    request: ChatRequest,
) -> Result<ChatResponse, String> {
    let ctx = state.backend_context()?;
    let app_handle = app.clone();

    ai::chat_with_graph_stream(&ctx, request, move |chunk| {
        let _ = app_handle.emit("chat-stream-chunk", chunk);
    })
    .await
}

#[tauri::command]
pub async fn suggest_tags(
    state: tauri::State<'_, AppState>,
    node_id: Uuid,
) -> Result<Vec<TagSuggestion>, String> {
    dispatch(
        &state,
        "suggest_tags",
        serde_json::to_value(node_id).map_err(|e| e.to_string())?,
    )
    .await
}
