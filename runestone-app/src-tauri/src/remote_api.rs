use crate::state::AppState;
use serde_json::Value;

pub async fn remote_request(
    state: &tauri::State<'_, AppState>,
    method: &str,
    path: &str,
    body: Option<Value>,
) -> Result<Value, String> {
    let config = state.get_remote_config().ok_or("No remote server configured")?;
    let (api_url, auth_token) = config;

    if api_url.is_empty() {
        return Err("No server URL configured. Go to Settings to connect to a Runestone server.".to_string());
    }

    let url = format!("{}{}", api_url.trim_end_matches('/'), path);
    let client = reqwest::Client::new();

    let mut req = match method.to_uppercase().as_str() {
        "GET" => client.get(&url),
        "POST" => {
            let mut r = client.post(&url);
            if let Some(b) = body {
                r = r.json(&b);
            }
            r
        }
        "PUT" => {
            let mut r = client.put(&url);
            if let Some(b) = body {
                r = r.json(&b);
            }
            r
        }
        "DELETE" => client.delete(&url),
        _ => return Err(format!("Unsupported HTTP method: {}", method)),
    };

    req = req.timeout(std::time::Duration::from_secs(30));
    req = req.header("Content-Type", "application/json");

    if let Some(token) = auth_token {
        req = req.header("Authorization", format!("Bearer {}", token));
    }

    let resp = req.send().await.map_err(|e| format!("Request failed: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!(
            "Server returned {}: {}",
            resp.status(),
            resp.text().await.unwrap_or_default()
        ));
    }

    let json: Value = resp.json().await.map_err(|e| format!("Response parse error: {}", e))?;
    Ok(json)
}
