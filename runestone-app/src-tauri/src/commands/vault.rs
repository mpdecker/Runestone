use crate::router::dispatch;
use crate::state::AppState;
use crate::vault_watcher;
use runestone_core::models::vault::{CreateVaultRequest, Vault};
use uuid::Uuid;

#[tauri::command]
pub async fn init_database(state: tauri::State<'_, AppState>) -> Result<String, String> {
    dispatch(&state, "init_database", serde_json::Value::Null).await
}

#[tauri::command]
pub async fn create_vault(
    state: tauri::State<'_, AppState>,
    request: CreateVaultRequest,
) -> Result<Vault, String> {
    dispatch(
        &state,
        "create_vault",
        serde_json::to_value(request).map_err(|e| e.to_string())?,
    )
    .await
}

#[tauri::command]
pub async fn list_vaults(state: tauri::State<'_, AppState>) -> Result<Vec<Vault>, String> {
    dispatch(&state, "list_vaults", serde_json::Value::Null).await
}

#[tauri::command]
pub async fn start_vault_watcher(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    vault_id: Uuid,
) -> Result<(), String> {
    if !state.has_local_pools() {
        return Ok(());
    }
    vault_watcher::start_vault_watcher(app, &state, vault_id)
}

#[tauri::command]
pub async fn stop_vault_watcher() -> Result<(), String> {
    vault_watcher::stop_vault_watcher()
}
