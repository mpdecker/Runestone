use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionNode {
    pub name: String,
    pub description: String,
    pub extraction_type: String,
    pub confidence: f64,
    pub source_node_id: Uuid,
    pub chunk_index: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PendingExtraction {
    pub id: Uuid,
    pub title: String,
    pub content_type: String,
    pub metadata: Option<serde_json::Value>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}
