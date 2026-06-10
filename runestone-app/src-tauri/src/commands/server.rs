use crate::state::{AppState, ConnectionMode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectionStatus {
    pub mode: String,
    pub api_url: Option<String>,
    pub connected: bool,
    pub local_db_available: bool,
}

#[tauri::command]
pub fn configure_server_connection(
    state: tauri::State<'_, AppState>,
    api_url: String,
    auth_token: Option<String>,
) -> Result<ConnectionStatus, String> {
    state.set_remote_config(api_url.clone(), auth_token);
    state.clear_local_pools();
    Ok(ConnectionStatus {
        mode: "remote".to_string(),
        api_url: Some(api_url),
        connected: false,
        local_db_available: false,
    })
}

#[tauri::command]
pub fn get_connection_status(
    state: tauri::State<'_, AppState>,
) -> Result<ConnectionStatus, String> {
    let mode = state.connection_mode.lock().map_err(|e| e.to_string())?;
    let mode_str = match &*mode {
        ConnectionMode::Local => "local".to_string(),
        ConnectionMode::Remote { api_url, .. } => {
            if api_url.is_empty() {
                "remote (not configured)".to_string()
            } else {
                format!("remote ({})", api_url)
            }
        }
    };

    let (api_url, connected) = match &*mode {
        ConnectionMode::Local => (None, true),
        ConnectionMode::Remote { api_url, .. } => {
            let has_url = !api_url.is_empty();
            (
                if has_url {
                    Some(api_url.clone())
                } else {
                    None
                },
                has_url && state.remote_connected(),
            )
        }
    };

    Ok(ConnectionStatus {
        mode: mode_str,
        api_url,
        connected,
        local_db_available: state.has_pg(),
    })
}

#[tauri::command]
pub async fn test_connection(state: tauri::State<'_, AppState>) -> Result<bool, String> {
    let config = state
        .get_remote_config()
        .ok_or("No remote server configured")?;
    let (api_url, auth_token) = config;

    if api_url.is_empty() {
        return Err("No server URL configured".to_string());
    }

    let client = reqwest::Client::new();
    let url = format!("{}/api/health", api_url.trim_end_matches('/'));

    let mut req = client.get(&url).timeout(std::time::Duration::from_secs(5));

    if let Some(token) = auth_token {
        req = req.header("Authorization", format!("Bearer {}", token));
    }

    match req.send().await {
        Ok(resp) => {
            if resp.status().is_success() {
                state.set_remote_connected(true);
                Ok(true)
            } else {
                state.set_remote_connected(false);
                Err(format!("Server returned status {}", resp.status()))
            }
        }
        Err(e) => {
            state.set_remote_connected(false);
            Err(format!("Connection failed: {}", e))
        }
    }
}
