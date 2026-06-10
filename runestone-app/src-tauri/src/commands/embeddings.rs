use crate::router::dispatch;
use crate::state::AppState;
use runestone_core::handlers::embeddings::EmbeddingStatus;
use uuid::Uuid;

#[tauri::command]
pub async fn reindex_vault(
    state: tauri::State<'_, AppState>,
    vault_id: Uuid,
) -> Result<serde_json::Value, String> {
    dispatch(
        &state,
        "reindex_vault",
        serde_json::to_value(vault_id).map_err(|e| e.to_string())?,
    )
    .await
}

#[tauri::command]
pub async fn get_embedding_status(
    state: tauri::State<'_, AppState>,
) -> Result<EmbeddingStatus, String> {
    dispatch(&state, "get_embedding_status", serde_json::Value::Null).await
}
