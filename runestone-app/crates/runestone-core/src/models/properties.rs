use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeProperty {
    pub key: String,
    pub value: Value,
    pub prop_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetPropertyRequest {
    pub node_id: Uuid,
    pub key: String,
    pub value: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertiesResponse {
    pub node_id: Uuid,
    pub properties: Vec<NodeProperty>,
}
