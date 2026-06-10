use crate::services::graph_sync;
use crate::state::AppState;
use crate::util::html_escape;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::Mutex;
use uuid::Uuid;

const MAX_CLIPPER_BODY_BYTES: usize = 1_048_576;

static SERVER_PORT: Mutex<Option<u16>> = Mutex::const_new(None);
static SERVER_RUNNING: AtomicBool = AtomicBool::new(false);

fn clipper_auth_token() -> String {
    std::env::var("CLIPPER_AUTH_TOKEN").unwrap_or_else(|_| "runestone-clipper-dev".to_string())
}

fn request_has_valid_token(headers: &str) -> bool {
    let expected = clipper_auth_token();
    headers.lines().any(|line| {
        line.to_lowercase().starts_with("x-clipper-token:") && line[16..].trim() == expected
    })
}

#[tauri::command]
pub async fn get_clipper_auth_token(state: tauri::State<'_, AppState>) -> Result<String, String> {
    crate::router::dispatch(&state, "get_clipper_auth_token", serde_json::Value::Null).await
}

pub async fn get_clipper_auth_token_impl() -> Result<String, String> {
    if !SERVER_RUNNING.load(Ordering::SeqCst) {
        return Err("Clipper server is not running".to_string());
    }
    Ok(clipper_auth_token())
}

#[tauri::command]
pub async fn start_clipper_server(
    state: tauri::State<'_, AppState>,
    vault_id: Uuid,
    port: Option<u16>,
) -> Result<u16, String> {
    crate::router::dispatch(
        &state,
        "start_clipper_server",
        serde_json::json!({ "vault_id": vault_id, "port": port }),
    )
    .await
}

pub async fn start_clipper_server_impl(
    state: &AppState,
    vault_id: Uuid,
    port: Option<u16>,
) -> Result<u16, String> {
    sqlx::query_scalar::<_, i64>("SELECT 1 FROM vaults WHERE id = $1")
        .bind(vault_id)
        .fetch_one(&state.pg()?)
        .await
        .map_err(|_| "Vault not found".to_string())?;

    let mut current = SERVER_PORT.lock().await;
    if current.is_some() {
        return Err("Clipper server already running".to_string());
    }

    let port = port.unwrap_or(9876);
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port))
        .map_err(|e| format!("Failed to bind: {}", e))?;

    let actual_port = listener.local_addr().map_err(|e| e.to_string())?.port();
    *current = Some(actual_port);
    drop(current);

    let pg = state.pg()?.clone();
    let neo4j = state.neo4j()?.clone();
    let runtime = tokio::runtime::Handle::current();

    SERVER_RUNNING.store(true, Ordering::SeqCst);

    std::thread::spawn(move || {
        for stream in listener.incoming() {
            if !SERVER_RUNNING.load(Ordering::SeqCst) {
                break;
            }

            if let Ok(mut stream) = stream {
                let _ = stream.set_read_timeout(Some(std::time::Duration::from_secs(5)));
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut request_line = String::new();
                if reader.read_line(&mut request_line).is_err() {
                    continue;
                }

                let mut headers = String::new();
                let mut content_length = 0usize;
                loop {
                    let mut line = String::new();
                    if reader.read_line(&mut line).is_err() {
                        break;
                    }
                    if line.trim().is_empty() {
                        break;
                    }
                    headers.push_str(&line);
                    if line.to_lowercase().starts_with("content-length:") {
                        content_length = line[15..].trim().parse().unwrap_or(0);
                    }
                }

                if request_line.starts_with("OPTIONS") {
                    let _ = stream.write_all(
                        b"HTTP/1.1 200 OK\r\nAccess-Control-Allow-Origin: http://localhost\r\nAccess-Control-Allow-Methods: POST, OPTIONS\r\nAccess-Control-Allow-Headers: Content-Type, X-Clipper-Token\r\nContent-Length: 0\r\n\r\n",
                    );
                    continue;
                }

                if request_line.starts_with("GET /health") {
                    if !request_has_valid_token(&headers) {
                        let _ = stream.write_all(
                            b"HTTP/1.1 401 Unauthorized\r\nContent-Length: 0\r\n\r\n",
                        );
                        continue;
                    }
                    let _ = stream.write_all(b"HTTP/1.1 200 OK\r\nAccess-Control-Allow-Origin: http://localhost\r\nContent-Type: application/json\r\nContent-Length: 20\r\n\r\n{\"status\":\"running\"}");
                    continue;
                }

                if !request_line.starts_with("POST /clip") {
                    continue;
                }

                if !request_has_valid_token(&headers) {
                    let _ = stream.write_all(
                        b"HTTP/1.1 401 Unauthorized\r\nContent-Length: 0\r\n\r\n",
                    );
                    continue;
                }

                if content_length > MAX_CLIPPER_BODY_BYTES {
                    let _ = stream.write_all(
                        b"HTTP/1.1 413 Payload Too Large\r\nContent-Length: 0\r\n\r\n",
                    );
                    continue;
                }

                let mut body = vec![0u8; content_length];
                if content_length > 0 && reader.read_exact(&mut body).is_err() {
                    continue;
                }

                let body_str = String::from_utf8_lossy(&body);
                let clip: serde_json::Value = match serde_json::from_str(&body_str) {
                    Ok(v) => v,
                    Err(_) => {
                        let _ = stream.write_all(
                            b"HTTP/1.1 400 Bad Request\r\n\r\nInvalid JSON",
                        );
                        continue;
                    }
                };

                let title = clip["title"].as_str().unwrap_or("Clipped Page").to_string();
                let url = clip["url"].as_str().unwrap_or("").to_string();
                let content = clip["content"].as_str().unwrap_or("").to_string();

                let safe_url = html_escape(&url);
                let safe_content = html_escape(&content);
                let formatted = format!(
                    "<p><a href=\"{}\">Source: {}</a></p><hr>{}",
                    safe_url, safe_url, safe_content
                );

                let pg_clone = pg.clone();
                let neo4j_clone = neo4j.clone();
                let id = Uuid::new_v4();
                let ftitle = title.clone();
                let fcontent = formatted;
                let wc = fcontent.split_whitespace().count() as i32;

                let result = runtime.block_on(async move {
                    sqlx::query(
                        "INSERT INTO nodes (id, vault_id, title, content, content_type, word_count) VALUES ($1, $2, $3, $4, 'note', $5)",
                    )
                    .bind(id)
                    .bind(vault_id)
                    .bind(&ftitle)
                    .bind(&fcontent)
                    .bind(wc)
                    .execute(&pg_clone)
                    .await
                    .map_err(|e| e.to_string())?;

                    graph_sync::create_node_with_pg_rollback(
                        &neo4j_clone,
                        &pg_clone,
                        id,
                        vault_id,
                        &ftitle,
                        "note",
                    )
                    .await?;

                    Ok::<(), String>(())
                });

                match result {
                    Ok(_) => {
                        let response = format!(
                            "HTTP/1.1 200 OK\r\nAccess-Control-Allow-Origin: http://localhost\r\nContent-Type: application/json\r\nContent-Length: 30\r\n\r\n{{\"status\":\"ok\",\"id\":\"{}\"}}",
                            id
                        );
                        let _ = stream.write_all(response.as_bytes());
                    }
                    Err(e) => {
                        let response = format!(
                            "HTTP/1.1 500 Internal Server Error\r\n\r\n{{\"error\":\"{}\"}}",
                            e
                        );
                        let _ = stream.write_all(response.as_bytes());
                    }
                }
            }
        }
    });

    Ok(actual_port)
}

#[tauri::command]
pub async fn stop_clipper_server(state: tauri::State<'_, AppState>) -> Result<(), String> {
    crate::router::dispatch(&state, "stop_clipper_server", serde_json::Value::Null).await
}

pub async fn stop_clipper_server_impl() -> Result<(), String> {
    SERVER_RUNNING.store(false, Ordering::SeqCst);
    let mut current = SERVER_PORT.lock().await;
    *current = None;
    Ok(())
}

#[tauri::command]
pub async fn get_clipper_status(state: tauri::State<'_, AppState>) -> Result<Option<u16>, String> {
    crate::router::dispatch(&state, "get_clipper_status", serde_json::Value::Null).await
}

pub async fn get_clipper_status_impl() -> Result<Option<u16>, String> {
    let current = SERVER_PORT.lock().await;
    Ok(*current)
}
