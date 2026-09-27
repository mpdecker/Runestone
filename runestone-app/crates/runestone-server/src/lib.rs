use axum::{
    extract::{DefaultBodyLimit, Path, State},
    http::{header::AUTHORIZATION, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use runestone_core::dispatch::dispatch_local;
use serde_json::Value;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;

pub const MAX_INVOKE_BODY_BYTES: usize = 2 * 1024 * 1024;

#[derive(Clone)]
pub struct ServerState {
    pub ctx: runestone_core::BackendContext,
    pub api_token: Option<String>,
    pub static_dir: Option<String>,
}

pub fn verify_bearer_token(headers: &axum::http::HeaderMap, expected: &str) -> bool {
    let auth = headers
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let token = auth.strip_prefix("Bearer ").unwrap_or("");
    constant_time_eq(token.as_bytes(), expected.as_bytes())
}

/// Compare without leaking, through timing, how many leading bytes of the token were right.
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    let mut diff = (a.len() ^ b.len()) as u8;
    for i in 0..a.len().max(b.len()) {
        diff |= a.get(i).copied().unwrap_or(0) ^ b.get(i).copied().unwrap_or(0);
    }
    diff == 0
}

pub fn build_router(state: Arc<ServerState>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let mut router = Router::new()
        .route("/api/health", get(health))
        .route("/api/invoke/{command}", axum::routing::post(invoke))
        .layer(DefaultBodyLimit::max(MAX_INVOKE_BODY_BYTES))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state.clone());

    if let Some(ref static_dir) = state.static_dir {
        let serve_dir = ServeDir::new(static_dir.clone())
            .precompressed_gzip()
            .precompressed_br()
            .fallback(ServeFile::new(format!("{}/index.html", static_dir)));

        router = router.fallback_service(serve_dir);
    }

    router
}

async fn health() -> Json<Value> {
    Json(serde_json::json!({ "status": "ok" }))
}

async fn invoke(
    State(state): State<Arc<ServerState>>,
    Path(command): Path<String>,
    headers: axum::http::HeaderMap,
    body: Json<Value>,
) -> Response {
    if let Some(ref expected) = state.api_token {
        if !verify_bearer_token(&headers, expected) {
            return (StatusCode::UNAUTHORIZED, "Invalid or missing API token").into_response();
        }
    }

    match dispatch_local(&state.ctx, &command, body.0).await {
        Ok(result) => Json(result).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, e).into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_bearer_token_accepts_valid_token() {
        let mut headers = axum::http::HeaderMap::new();
        headers.insert(AUTHORIZATION, "Bearer secret-token".parse().unwrap());
        assert!(verify_bearer_token(&headers, "secret-token"));
    }

    #[test]
    fn constant_time_eq_matches_only_equal_inputs() {
        assert!(constant_time_eq(b"abc", b"abc"));
        assert!(!constant_time_eq(b"abc", b"abd"));
        assert!(!constant_time_eq(b"abc", b"abcd"));
        assert!(!constant_time_eq(b"", b"a"));
        assert!(constant_time_eq(b"", b""));
    }

    #[test]
    fn verify_bearer_token_rejects_missing_or_invalid() {
        let headers = axum::http::HeaderMap::new();
        assert!(!verify_bearer_token(&headers, "secret-token"));

        let mut bad = axum::http::HeaderMap::new();
        bad.insert(AUTHORIZATION, "Bearer wrong".parse().unwrap());
        assert!(!verify_bearer_token(&bad, "secret-token"));
    }
}
