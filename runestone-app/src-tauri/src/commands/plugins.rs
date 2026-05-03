use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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
    plugin_dir: String,
) -> Result<Vec<PluginInfo>, String> {
    let dir = std::path::Path::new(&plugin_dir);
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut plugins = Vec::new();

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() { continue; }

            let manifest_path = path.join("manifest.json");
            if !manifest_path.exists() { continue; }

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
pub async fn read_plugin_file(path: String) -> Result<String, String> {
    std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read plugin file: {}", e))
}
