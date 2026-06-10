use crate::commands::{clipper, document, export, node, obsidian, plugins};
use crate::state::AppState;
use serde_json::Value;
use uuid::Uuid;

pub async fn dispatch_desktop_only(
    state: &AppState,
    command: &str,
    args: Value,
) -> Result<Value, String> {
    match command {
        "start_clipper_server" => {
            let vault_id: Uuid = parse_field(&args, "vault_id")?;
            let port: Option<u16> = parse_optional_field(&args, "port")?;
            let result = clipper::start_clipper_server_impl(state, vault_id, port).await?;
            Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
        }
        "stop_clipper_server" => {
            clipper::stop_clipper_server_impl().await?;
            Ok(Value::Null)
        }
        "get_clipper_status" => {
            let result = clipper::get_clipper_status_impl().await?;
            Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
        }
        "get_clipper_auth_token" => {
            let result = clipper::get_clipper_auth_token_impl().await?;
            Ok(Value::String(result))
        }
        "scan_vault" => {
            let vault_id: Uuid = parse_field(&args, "vault_id")?;
            let delete_orphans: bool = parse_optional_field(&args, "delete_orphans")?
                .unwrap_or(false);
            let result = node::scan_vault_impl(state, vault_id, delete_orphans).await?;
            Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
        }
        "import_document" => {
            let vault_id: Uuid = parse_field(&args, "vault_id")?;
            let file_path: String = parse_field(&args, "file_path")?;
            let result = document::import_document_impl(state, vault_id, file_path).await?;
            Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
        }
        "import_obsidian_vault" => {
            let vault_id: Uuid = parse_field(&args, "vault_id")?;
            let root_path: String = parse_field(&args, "root_path")?;
            let result =
                obsidian::import_obsidian_vault_impl(state, vault_id, root_path).await?;
            Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
        }
        "export_node_to_markdown" => {
            let node_id: Uuid = parse_field(&args, "node_id")?;
            let export_path: Option<String> = parse_optional_field(&args, "export_path")?;
            let result =
                export::export_node_to_markdown_impl(state, node_id, export_path).await?;
            Ok(Value::String(result))
        }
        "export_vault_to_markdown" => {
            let vault_id: Uuid = parse_field(&args, "vault_id")?;
            let output_dir: String = parse_field(&args, "output_dir")?;
            let result = export::export_vault_to_markdown_impl(state, vault_id, output_dir).await?;
            Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
        }
        "list_available_plugins" => {
            let plugin_dir: String = parse_field(&args, "plugin_dir")?;
            let result = plugins::list_available_plugins_impl(plugin_dir).await?;
            Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
        }
        "read_plugin_file" => {
            let plugin_root: String = parse_field(&args, "plugin_root")?;
            let relative_path: String = parse_field(&args, "relative_path")?;
            let result =
                plugins::read_plugin_file_impl(plugin_root, relative_path).await?;
            Ok(Value::String(result))
        }
        other => Err(format!("Unknown desktop-only command: {}", other)),
    }
}

fn parse_uuid(args: &Value) -> Result<Uuid, String> {
    if let Some(s) = args.as_str() {
        return Uuid::parse_str(s).map_err(|e| e.to_string());
    }
    serde_json::from_value(args.clone()).map_err(|e| e.to_string())
}

fn parse_field<T: serde::de::DeserializeOwned>(args: &Value, field: &str) -> Result<T, String> {
    if let Some(obj) = args.as_object() {
        if let Some(val) = obj.get(field) {
            return serde_json::from_value(val.clone()).map_err(|e| e.to_string());
        }
    }
    serde_json::from_value(args.clone()).map_err(|e| e.to_string())
}

fn parse_optional_field<T: serde::de::DeserializeOwned>(
    args: &Value,
    field: &str,
) -> Result<Option<T>, String> {
    if let Some(obj) = args.as_object() {
        if let Some(val) = obj.get(field) {
            if val.is_null() {
                return Ok(None);
            }
            return serde_json::from_value(val.clone())
                .map(Some)
                .map_err(|e| e.to_string());
        }
    }
    Ok(None)
}
