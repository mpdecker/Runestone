use crate::router::dispatch;
use crate::state::AppState;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: Option<String>,
    pub path: String,
    pub main_file: String,
}

#[tauri::command]
pub async fn list_available_plugins(
    state: tauri::State<'_, AppState>,
    plugin_dir: String,
) -> Result<Vec<PluginInfo>, String> {
    dispatch(
        &state,
        "list_available_plugins",
        serde_json::json!({ "plugin_dir": plugin_dir }),
    )
    .await
}

pub async fn list_available_plugins_impl(plugin_dir: String) -> Result<Vec<PluginInfo>, String> {
    let dir = std::path::Path::new(&plugin_dir);
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut plugins = Vec::new();

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let manifest_path = path.join("manifest.json");
            if !manifest_path.exists() {
                continue;
            }

            let content = std::fs::read_to_string(&manifest_path)
                .map_err(|e| format!("Failed to read manifest: {}", e))?;

            let plugin: PluginInfo = serde_json::from_str(&content)
                .map_err(|e| format!("Invalid manifest: {}", e))?;

            plugins.push(PluginInfo {
                path: path.to_string_lossy().to_string(),
                ..plugin
            });
        }
    }

    Ok(plugins)
}

#[tauri::command]
pub async fn read_plugin_file(
    state: tauri::State<'_, AppState>,
    plugin_root: String,
    relative_path: String,
) -> Result<String, String> {
    dispatch(
        &state,
        "read_plugin_file",
        serde_json::json!({
            "plugin_root": plugin_root,
            "relative_path": relative_path,
        }),
    )
    .await
}

pub async fn read_plugin_file_impl(
    plugin_root: String,
    relative_path: String,
) -> Result<String, String> {
    let root = std::path::Path::new(&plugin_root)
        .canonicalize()
        .map_err(|e| format!("Invalid plugin root: {}", e))?;

    let joined = root.join(&relative_path);
    let canonical = joined
        .canonicalize()
        .map_err(|e| format!("Invalid plugin file path: {}", e))?;

    if !canonical.starts_with(&root) {
        return Err("Plugin file must be within plugin directory".to_string());
    }

    std::fs::read_to_string(&canonical)
        .map_err(|e| format!("Failed to read plugin file: {}", e))
}
