use crate::models::extraction::PendingExtraction;
use crate::state::AppState;
use uuid::Uuid;

#[tauri::command]
pub async fn get_pending_extractions(
    state: tauri::State<'_, AppState>,
    vault_id: Uuid,
) -> Result<Vec<PendingExtraction>, String> {
    let extractions = sqlx::query_as::<_, PendingExtraction>(
        r#"SELECT id, title, content_type, metadata, created_at
           FROM nodes
           WHERE vault_id = $1
             AND (content_type = 'entity' OR content_type = 'concept')
             AND metadata->>'status' = 'pending'
           ORDER BY created_at DESC"#,
    )
    .bind(vault_id)
    .fetch_all(state.pg()?)
    .await
    .map_err(|e| format!("Failed to get pending extractions: {}", e))?;

    Ok(extractions)
}

#[tauri::command]
pub async fn approve_extraction(
    state: tauri::State<'_, AppState>,
    extraction_id: Uuid,
) -> Result<(), String> {
    let metadata = serde_json::json!({"status": "approved"});

    sqlx::query("UPDATE nodes SET metadata = metadata || $1 WHERE id = $2")
        .bind(&metadata)
        .bind(extraction_id)
        .execute(state.pg()?)
        .await
        .map_err(|e| format!("Failed to approve extraction: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn reject_extraction(
    state: tauri::State<'_, AppState>,
    extraction_id: Uuid,
) -> Result<(), String> {
    let metadata = serde_json::json!({"status": "rejected"});

    sqlx::query("UPDATE nodes SET metadata = metadata || $1 WHERE id = $2")
        .bind(&metadata)
        .bind(extraction_id)
        .execute(state.pg()?)
        .await
        .map_err(|e| format!("Failed to reject extraction: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn batch_approve_extractions(
    state: tauri::State<'_, AppState>,
    extraction_ids: Vec<Uuid>,
) -> Result<(), String> {
    let metadata = serde_json::json!({"status": "approved"});

    for id in &extraction_ids {
        sqlx::query("UPDATE nodes SET metadata = metadata || $1 WHERE id = $2")
            .bind(&metadata)
            .bind(id)
            .execute(state.pg()?)
            .await
            .map_err(|e| format!("Failed to approve extraction {}: {}", id, e))?;
    }

    Ok(())
}
