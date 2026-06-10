use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SearchResult {
    pub node_id: Uuid,
    pub title: String,
    pub content_type: String,
    pub snippet: String,
    pub score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResults {
    pub vector_results: Vec<SearchResult>,
    pub fts_results: Vec<SearchResult>,
    pub combined: Vec<SearchResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub vault_id: Uuid,
    pub query: String,
    pub limit: Option<i64>,
    pub include_fts: Option<bool>,
}
