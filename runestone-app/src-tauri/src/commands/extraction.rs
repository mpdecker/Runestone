use crate::router::dispatch;
use crate::models::extraction::PendingExtraction;
use crate::state::AppState;
use uuid::Uuid;

#[tauri::command]
pub async fn get_pending_extractions(
    state: tauri::State<'_, AppState>,
    vault_id: Uuid,
) -> Result<Vec<PendingExtraction>, String> {
    dispatch(
        &state,
        "get_pending_extractions",
        serde_json::to_value(vault_id).map_err(|e| e.to_string())?,
    )
    .await
}

#[tauri::command]
pub async fn approve_extraction(
    state: tauri::State<'_, AppState>,
    extraction_id: Uuid,
) -> Result<(), String> {
    dispatch(
        &state,
        "approve_extraction",
        serde_json::to_value(extraction_id).map_err(|e| e.to_string())?,
    )
    .await
}

#[tauri::command]
pub async fn reject_extraction(
    state: tauri::State<'_, AppState>,
    extraction_id: Uuid,
) -> Result<(), String> {
    dispatch(
        &state,
        "reject_extraction",
        serde_json::to_value(extraction_id).map_err(|e| e.to_string())?,
    )
    .await
}

#[tauri::command]
pub async fn batch_approve_extractions(
    state: tauri::State<'_, AppState>,
    extraction_ids: Vec<Uuid>,
) -> Result<(), String> {
    dispatch(
        &state,
        "batch_approve_extractions",
        serde_json::to_value(extraction_ids).map_err(|e| e.to_string())?,
    )
    .await
}
