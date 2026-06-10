use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct NodeVersion {
    pub id: Uuid,
    pub node_id: Uuid,
    pub version_number: i32,
    pub title: String,
    pub content: String,
    pub word_count: i32,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}
