use axum::{
    extract::{DefaultBodyLimit, Path, State},
    http::{header::AUTHORIZATION, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use runestone_core::dispatch::dispatch_local;
use runestone_core::BackendContext;
use serde_json::Value;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

pub const MAX_INVOKE_BODY_BYTES: usize = 2 * 1024 * 1024;

#[derive(Clone)]
pub struct ServerState {
    pub ctx: BackendContext,
    pub api_token: Option<String>,
}

pub fn verify_bearer_token(headers: &axum::http::HeaderMap, expected: &str) -> bool {
    let auth = headers
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let token = auth.strip_prefix("Bearer ").unwrap_or("");
    token == expected
}

pub fn build_router(state: Arc<ServerState>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/api/health", get(health))
        .route("/api/invoke/{command}", post(invoke))
        .layer(DefaultBodyLimit::max(MAX_INVOKE_BODY_BYTES))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
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
        headers.insert(
            AUTHORIZATION,
            "Bearer secret-token".parse().unwrap(),
        );
        assert!(verify_bearer_token(&headers, "secret-token"));
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
