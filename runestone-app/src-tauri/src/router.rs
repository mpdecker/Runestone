use crate::desktop_dispatch;
use crate::remote_api;
use crate::state::AppState;
use runestone_core::dispatch::{dispatch_local, is_desktop_only};
use runestone_core::AppError;
use serde::de::DeserializeOwned;
use serde_json::Value;

pub async fn dispatch<T: DeserializeOwned>(
    state: &AppState,
    command: &str,
    args: Value,
) -> Result<T, String> {
    if is_desktop_only(command) {
        if !state.is_local() && state.is_remote_configured() {
            return Err(AppError::DesktopOnly(command.to_string()).to_string());
        }
        if !state.has_local_pools() {
            return Err(AppError::DesktopOnly(command.to_string()).to_string());
        }
        let result = desktop_dispatch::dispatch_desktop_only(state, command, args).await?;
        return serde_json::from_value(result).map_err(|e| e.to_string());
    }

    if state.is_local() && state.has_local_pools() {
        let ctx = state.backend_context()?;
        let result = dispatch_local(&ctx, command, args).await?;
        return serde_json::from_value(result).map_err(|e| e.to_string());
    }

    if state.is_remote_configured() {
        return remote_api::remote_invoke(state, command, args).await;
    }

    Err("No database connection available. Configure a local database or connect to a remote server.".to_string())
}
