use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Node {
    pub id: Uuid,
    pub vault_id: Uuid,
    pub title: String,
    pub content: String,
    pub content_type: String,
    pub file_path: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub word_count: i32,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateNodeRequest {
    pub vault_id: Uuid,
    pub title: String,
    pub content: String,
    pub content_type: Option<String>,
    pub file_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateNodeRequest {
    pub id: Uuid,
    pub title: Option<String>,
    pub content: Option<String>,
    pub content_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct NodeListItem {
    pub id: Uuid,
    pub title: String,
    pub content_type: String,
    pub file_path: Option<String>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct NodeIdRow {
    pub id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanVaultResult {
    pub created: u32,
    pub updated: u32,
    pub skipped: u32,
    pub deleted: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListNodesRequest {
    pub vault_id: Uuid,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
