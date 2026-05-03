use crate::state::AppState;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

static SERVER_PORT: Mutex<Option<u16>> = Mutex::const_new(None);
static SERVER_RUNNING: AtomicBool = AtomicBool::new(false);

#[tauri::command]
pub async fn start_clipper_server(
    state: tauri::State<'_, AppState>,
    vault_id: Uuid,
    port: Option<u16>,
) -> Result<u16, String> {
    let mut current = SERVER_PORT.lock().await;
    if current.is_some() {
        return Err("Clipper server already running. Port: {}".to_string());
    }

    let port = port.unwrap_or(9876);
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port))
        .map_err(|e| format!("Failed to bind: {}", e))?;

    let actual_port = listener.local_addr().map_err(|e| e.to_string())?.port();
    *current = Some(actual_port);
    drop(current);

    let pg = state.pg()?.clone();
    let neo4j = state.neo4j()?.clone();

    SERVER_RUNNING.store(true, Ordering::SeqCst);

    std::thread::spawn(move || {
        for stream in listener.incoming() {
            if !SERVER_RUNNING.load(Ordering::SeqCst) { break; }

            if let Ok(mut stream) = stream {
                let _ = stream.set_read_timeout(Some(std::time::Duration::from_secs(5)));
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut request_line = String::new();
                if reader.read_line(&mut request_line).is_err() { continue; }

                let mut content_length = 0usize;
                loop {
                    let mut line = String::new();
                    if reader.read_line(&mut line).is_err() { break; }
                    if line.trim().is_empty() { break; }
                    if line.to_lowercase().starts_with("content-length:") {
                        content_length = line[15..].trim().parse().unwrap_or(0);
                    }
                }

                let mut body = vec![0u8; content_length];
                if content_length > 0 {
                    let _ = reader.read_exact(&mut body);
                }

                if request_line.starts_with("OPTIONS") {
                    let _ = stream.write_all(b"HTTP/1.1 200 OK\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: POST, OPTIONS\r\nAccess-Control-Allow-Headers: Content-Type\r\nContent-Length: 0\r\n\r\n");
                    continue;
                }

                if request_line.starts_with("GET /health") {
                    let _ = stream.write_all(b"HTTP/1.1 200 OK\r\nAccess-Control-Allow-Origin: *\r\nContent-Type: application/json\r\nContent-Length: 20\r\n\r\n{\"status\":\"running\"}");
                    continue;
                }

                if !request_line.starts_with("POST /clip") { continue; }

                let body_str = String::from_utf8_lossy(&body);
                let clip: serde_json::Value = match serde_json::from_str(&body_str) {
                    Ok(v) => v,
                    Err(_) => {
                        let _ = stream.write_all(b"HTTP/1.1 400 Bad Request\r\n\r\nInvalid JSON");
                        continue;
                    }
                };

                let title = clip["title"].as_str().unwrap_or("Clipped Page").to_string();
                let url = clip["url"].as_str().unwrap_or("").to_string();
                let content = clip["content"].as_str().unwrap_or("").to_string();

                let formatted = format!(
                    "<p><a href=\"{}\">Source: {}</a></p><hr>{}",
                    url, url, content
                );

                let pg_clone = pg.clone();
                let neo4j_clone = neo4j.clone();
                let handle = tokio::runtime::Handle::current();
                let id = Uuid::new_v4();
                let fid = id;
                let ftitle = title.clone();
                let fcontent = formatted;
                let wc = fcontent.split_whitespace().count() as i32;

                let result = handle.block_on(async move {
                    sqlx::query(
                        "INSERT INTO nodes (id, vault_id, title, content, content_type, word_count) VALUES ($1, $2, $3, $4, 'note', $5)"
                    )
                    .bind(fid)
                    .bind(vault_id)
                    .bind(&ftitle)
                    .bind(&fcontent)
                    .bind(wc)
                    .execute(&pg_clone)
                    .await
                });

                match result {
                    Ok(_) => {
                        let n4j = neo4j_clone;
                        let response = format!("HTTP/1.1 200 OK\r\nAccess-Control-Allow-Origin: *\r\nContent-Type: application/json\r\nContent-Length: 30\r\n\r\n{{\"status\":\"ok\",\"id\":\"{}\"}}", id);
                        let _ = stream.write_all(response.as_bytes());
                        if let Ok(handle) = tokio::runtime::Handle::try_current() {
                            handle.spawn(async move {
                                let _ = n4j.run(neo4rs::query(
                                    "CREATE (n:Node {pg_id: $pg_id, vault_id: $vault_id, title: $title, content_type: 'note'})"
                                )
                                .param("pg_id", id.to_string())
                                .param("vault_id", vault_id.to_string())
                                .param("title", title)).await;
                            });
                        }
                    }
                    Err(e) => {
                        let response = format!("HTTP/1.1 500 Internal Server Error\r\n\r\n{{\"error\":\"{}\"}}", e);
                        let _ = stream.write_all(response.as_bytes());
                    }
                }
            }
        }
    });

    Ok(actual_port)
}

#[tauri::command]
pub async fn stop_clipper_server() -> Result<(), String> {
    SERVER_RUNNING.store(false, Ordering::SeqCst);
    let mut current = SERVER_PORT.lock().await;
    *current = None;
    Ok(())
}

#[tauri::command]
pub async fn get_clipper_status() -> Result<Option<u16>, String> {
    let current = SERVER_PORT.lock().await;
    Ok(*current)
}
