use crate::state::AppState;
use serde::de::DeserializeOwned;
use serde_json::Value;

pub async fn remote_invoke<T: DeserializeOwned>(
    state: &AppState,
    command: &str,
    args: Value,
) -> Result<T, String> {
    let config = state
        .get_remote_config()
        .ok_or("No remote server configured")?;
    let (api_url, auth_token) = config;

    if api_url.is_empty() {
        return Err(
            "No server URL configured. Go to Settings to connect to a Runestone server.".to_string(),
        );
    }

    let url = format!(
        "{}/api/invoke/{}",
        api_url.trim_end_matches('/'),
        command
    );
    let client = reqwest::Client::new();

    let mut req = client
        .post(&url)
        .json(&args)
        .timeout(std::time::Duration::from_secs(60))
        .header("Content-Type", "application/json");

    if let Some(token) = auth_token {
        req = req.header("Authorization", format!("Bearer {}", token));
    }

    let resp = req
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!(
            "Server returned {}: {}",
            resp.status(),
            resp.text().await.unwrap_or_default()
        ));
    }

    let json: Value = resp
        .json()
        .await
        .map_err(|e| format!("Response parse error: {}", e))?;

    serde_json::from_value(json).map_err(|e| format!("Deserialize error: {}", e))
}
