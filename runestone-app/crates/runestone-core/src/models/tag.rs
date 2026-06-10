use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddTagsRequest {
    pub node_id: Uuid,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveTagRequest {
    pub node_id: Uuid,
    pub tag: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TagInfo {
    pub name: String,
    pub node_count: Option<i64>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TaggedNode {
    pub node_id: Uuid,
    pub title: String,
    pub content_type: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagsResponse {
    pub node_id: Uuid,
    pub tags: Vec<String>,
}
