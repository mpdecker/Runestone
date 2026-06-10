use crate::models::vault::Vault;
use crate::state::AppState;
use notify::RecursiveMode;
use notify_debouncer_mini::{new_debouncer, DebounceEventResult};
use std::path::Path;
use std::sync::Mutex;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use uuid::Uuid;

static WATCHER: Mutex<Option<notify_debouncer_mini::Debouncer<notify::RecommendedWatcher>>> =
    Mutex::new(None);

pub fn start_vault_watcher(
    app: AppHandle,
    state: &AppState,
    vault_id: Uuid,
) -> Result<(), String> {
    stop_vault_watcher()?;

    let vault = tauri::async_runtime::block_on(async {
        sqlx::query_as::<_, Vault>(
            "SELECT id, name, root_path, created_at, updated_at FROM vaults WHERE id = $1",
        )
        .bind(vault_id)
        .fetch_one(&state.pg()?)
        .await
        .map_err(|e| format!("Vault not found: {}", e))
    })?;

    let watch_path = vault.root_path.clone();
    let app_clone = app.clone();

    let mut debouncer = new_debouncer(
        Duration::from_millis(500),
        move |result: DebounceEventResult| {
            if let Ok(events) = result {
                for event in events {
                    let path = &event.path;
                    if path.extension().is_some_and(|e| e == "md") {
                        let payload = serde_json::json!({
                            "vault_id": vault_id.to_string(),
                            "file_path": path.to_string_lossy().to_string(),
                            "kind": format!("{:?}", event.kind),
                        });
                        let _ = app_clone.emit("vault-file-changed", payload);
                    }
                }
            }
        },
    )
    .map_err(|e| format!("Failed to create debouncer: {}", e))?;

    debouncer
        .watcher()
        .watch(Path::new(&watch_path), RecursiveMode::Recursive)
        .map_err(|e| format!("Failed to watch vault: {}", e))?;

    if let Ok(mut guard) = WATCHER.lock() {
        *guard = Some(debouncer);
    }

    log::info!("Vault watcher started for {}", watch_path);
    Ok(())
}

pub fn stop_vault_watcher() -> Result<(), String> {
    if let Ok(mut guard) = WATCHER.lock() {
        *guard = None;
    }
    Ok(())
}
