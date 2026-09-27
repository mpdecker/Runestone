use crate::context::BackendContext;
use crate::models::properties::{NodeProperty, PropertiesResponse, SetPropertyRequest};
use serde_json::Value;
use uuid::Uuid;

/// Metadata keys owned by other features (tags, aliases, AI extraction bookkeeping). They are
/// hidden from the property list, so letting a user write them through the property editor
/// silently swallows the value - and writing a non-array to `tags` used to break `list_tags`
/// for the whole vault.
const RESERVED_KEYS: [&str; 7] = [
    "tags",
    "aliases",
    "status",
    "confidence",
    "source_chunk",
    "description",
    "extraction_type",
];

fn validate_key(key: &str) -> Result<(), String> {
    if key.trim().is_empty() {
        return Err("Property name must not be empty".to_string());
    }
    if RESERVED_KEYS.contains(&key) {
        return Err(format!("'{}' is a reserved property name", key));
    }
    Ok(())
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
    let reserved = RESERVED_KEYS;

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

pub async fn get_node_properties(
    ctx: &BackendContext,
    node_id: Uuid,
) -> Result<PropertiesResponse, String> {
    let row = sqlx::query_as::<_, (Option<Value>,)>("SELECT metadata FROM nodes WHERE id = $1")
        .bind(node_id)
        .fetch_one(&ctx.pg)
        .await
        .map_err(|e| format!("Node not found: {}", e))?;

    let metadata = row.0.unwrap_or(Value::Object(Default::default()));
    let properties = extract_properties(&metadata);

    Ok(PropertiesResponse {
        node_id,
        properties,
    })
}

pub async fn set_node_property(
    ctx: &BackendContext,
    request: SetPropertyRequest,
) -> Result<PropertiesResponse, String> {
    validate_key(&request.key)?;
    let row = sqlx::query_as::<_, (Option<Value>,)>("SELECT metadata FROM nodes WHERE id = $1")
        .bind(request.node_id)
        .fetch_one(&ctx.pg)
        .await
        .map_err(|e| format!("Node not found: {}", e))?;

    let mut meta = row
        .0
        .filter(Value::is_object)
        .unwrap_or(serde_json::json!({}));
    meta[&request.key] = request.value;

    sqlx::query("UPDATE nodes SET metadata = $2, updated_at = NOW() WHERE id = $1")
        .bind(request.node_id)
        .bind(&meta)
        .execute(&ctx.pg)
        .await
        .map_err(|e| format!("Failed to update property: {}", e))?;

    let properties = extract_properties(&meta);

    Ok(PropertiesResponse {
        node_id: request.node_id,
        properties,
    })
}

pub async fn remove_node_property(
    ctx: &BackendContext,
    node_id: Uuid,
    key: String,
) -> Result<PropertiesResponse, String> {
    let row = sqlx::query_as::<_, (Option<Value>,)>("SELECT metadata FROM nodes WHERE id = $1")
        .bind(node_id)
        .fetch_one(&ctx.pg)
        .await
        .map_err(|e| format!("Node not found: {}", e))?;

    if RESERVED_KEYS.contains(&key.as_str()) {
        return Err(format!("'{}' is a reserved property name", key));
    }
    let mut meta = row
        .0
        .filter(Value::is_object)
        .unwrap_or(serde_json::json!({}));
    if let Some(obj) = meta.as_object_mut() {
        obj.remove(&key);
    }

    sqlx::query("UPDATE nodes SET metadata = $2, updated_at = NOW() WHERE id = $1")
        .bind(node_id)
        .bind(&meta)
        .execute(&ctx.pg)
        .await
        .map_err(|e| format!("Failed to remove property: {}", e))?;

    let properties = extract_properties(&meta);

    Ok(PropertiesResponse {
        node_id,
        properties,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reserved_and_empty_keys_are_rejected() {
        assert!(validate_key("tags").is_err());
        assert!(validate_key("aliases").is_err());
        assert!(validate_key("status").is_err());
        assert!(validate_key("  ").is_err());
        assert!(validate_key("author").is_ok());
    }
}
