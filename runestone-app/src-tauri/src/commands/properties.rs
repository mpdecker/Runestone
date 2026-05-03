use crate::state::AppState;
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

fn infer_type(value: &Value) -> &str {
    match value {
        Value::String(_) => "text",
        Value::Number(_) => "number",
        Value::Bool(_) => "checkbox",
        Value::Array(_) => "list",
        _ => "text",
    }
}

fn extract_properties(metadata: &Value) -> Vec<NodeProperty> {
    let mut properties: Vec<NodeProperty> = Vec::new();
    let reserved = ["tags", "status", "confidence", "source_chunk", "description", "extraction_type"];

    if let Some(obj) = metadata.as_object() {
        for (key, value) in obj {
            if reserved.contains(&key.as_str()) {
                continue;
            }
            let prop_type = infer_type(value);
            properties.push(NodeProperty {
                key: key.clone(),
                value: value.clone(),
                prop_type: prop_type.to_string(),
            });
        }
    }

    properties
}

#[tauri::command]
pub async fn get_node_properties(
    state: tauri::State<'_, AppState>,
    node_id: Uuid,
) -> Result<PropertiesResponse, String> {
    let row = sqlx::query_as::<_, (Option<Value>,)>(
        "SELECT metadata FROM nodes WHERE id = $1",
    )
    .bind(node_id)
    .fetch_one(state.pg()?)
    .await
    .map_err(|e| format!("Node not found: {}", e))?;

    let metadata = row.0.unwrap_or(Value::Object(Default::default()));
    let properties = extract_properties(&metadata);

    Ok(PropertiesResponse {
        node_id,
        properties,
    })
}

#[tauri::command]
pub async fn set_node_property(
    state: tauri::State<'_, AppState>,
    request: SetPropertyRequest,
) -> Result<PropertiesResponse, String> {
    let row = sqlx::query_as::<_, (Option<Value>,)>(
        "SELECT metadata FROM nodes WHERE id = $1",
    )
    .bind(request.node_id)
    .fetch_one(state.pg()?)
    .await
    .map_err(|e| format!("Node not found: {}", e))?;

    let mut meta = row.0.unwrap_or(serde_json::json!({}));
    meta[&request.key] = request.value;

    sqlx::query("UPDATE nodes SET metadata = $2, updated_at = NOW() WHERE id = $1")
        .bind(request.node_id)
        .bind(&meta)
        .execute(state.pg()?)
        .await
        .map_err(|e| format!("Failed to update property: {}", e))?;

    let properties = extract_properties(&meta);

    Ok(PropertiesResponse {
        node_id: request.node_id,
        properties,
    })
}

#[tauri::command]
pub async fn remove_node_property(
    state: tauri::State<'_, AppState>,
    node_id: Uuid,
    key: String,
) -> Result<PropertiesResponse, String> {
    let row = sqlx::query_as::<_, (Option<Value>,)>(
        "SELECT metadata FROM nodes WHERE id = $1",
    )
    .bind(node_id)
    .fetch_one(state.pg()?)
    .await
    .map_err(|e| format!("Node not found: {}", e))?;

    let mut meta = row.0.unwrap_or(serde_json::json!({}));
    if let Some(obj) = meta.as_object_mut() {
        obj.remove(&key);
    }

    sqlx::query("UPDATE nodes SET metadata = $2, updated_at = NOW() WHERE id = $1")
        .bind(node_id)
        .bind(&meta)
        .execute(state.pg()?)
        .await
        .map_err(|e| format!("Failed to remove property: {}", e))?;

    let properties = extract_properties(&meta);

    Ok(PropertiesResponse {
        node_id,
        properties,
    })
}
