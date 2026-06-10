use crate::router::dispatch;
use crate::models::node::{CreateNodeRequest, Node, NodeListItem, ScanVaultResult, UpdateNodeRequest};
use crate::models::vault::Vault;
use crate::path_guard::canonicalize_path;
use crate::repositories::node_repo;
use crate::services::vault_sync::{self, UpsertAction};
use crate::state::AppState;
use std::collections::HashSet;
use uuid::Uuid;

#[tauri::command]
pub async fn create_node(
    state: tauri::State<'_, AppState>,
    request: CreateNodeRequest,
) -> Result<Node, String> {
    dispatch(
        &state,
        "create_node",
        serde_json::to_value(request).map_err(|e| e.to_string())?,
    )
    .await
}

#[tauri::command]
pub async fn update_node(
    state: tauri::State<'_, AppState>,
    request: UpdateNodeRequest,
) -> Result<Node, String> {
    dispatch(
        &state,
        "update_node",
        serde_json::to_value(request).map_err(|e| e.to_string())?,
    )
    .await
}

#[tauri::command]
pub async fn delete_node(state: tauri::State<'_, AppState>, id: Uuid) -> Result<(), String> {
    dispatch(
        &state,
        "delete_node",
        serde_json::to_value(id).map_err(|e| e.to_string())?,
    )
    .await
}

#[tauri::command]
pub async fn get_node(state: tauri::State<'_, AppState>, id: Uuid) -> Result<Node, String> {
    dispatch(
        &state,
        "get_node",
        serde_json::to_value(id).map_err(|e| e.to_string())?,
    )
    .await
}

#[tauri::command]
pub async fn list_nodes(
    state: tauri::State<'_, AppState>,
    vault_id: Uuid,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<NodeListItem>, String> {
    dispatch(
        &state,
        "list_nodes",
        serde_json::json!({ "vault_id": vault_id, "limit": limit, "offset": offset }),
    )
    .await
}

#[tauri::command]
pub async fn scan_vault(
    state: tauri::State<'_, AppState>,
    vault_id: Uuid,
    delete_orphans: Option<bool>,
) -> Result<ScanVaultResult, String> {
    dispatch(
        &state,
        "scan_vault",
        serde_json::json!({ "vault_id": vault_id, "delete_orphans": delete_orphans }),
    )
    .await
}

pub async fn scan_vault_impl(
    state: &AppState,
    vault_id: Uuid,
    delete_orphans: bool,
) -> Result<ScanVaultResult, String> {
    let vault = sqlx::query_as::<_, Vault>(
        "SELECT id, name, root_path, created_at, updated_at FROM vaults WHERE id = $1",
    )
    .bind(vault_id)
    .fetch_one(&state.pg()?)
    .await
    .map_err(|e| format!("Vault not found: {}", e))?;

    let scan_root = canonicalize_path(&vault.root_path).map_err(|e| e.to_string())?;
    let pool = state.pg()?;
    let graph = state.neo4j()?;

    let mut result = ScanVaultResult {
        created: 0,
        updated: 0,
        skipped: 0,
        deleted: 0,
    };

    let mut seen_paths: HashSet<String> = HashSet::new();

    for entry in walkdir::WalkDir::new(&scan_root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
    {
        let file_path = entry.path().to_string_lossy().to_string();
        seen_paths.insert(file_path.clone());

        let content = std::fs::read_to_string(entry.path()).unwrap_or_default();
        let title = entry
            .path()
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        match vault_sync::upsert_from_file(&pool, &graph, vault_id, &file_path, &title, &content)
            .await
        {
            Ok((_, action)) => match action {
                UpsertAction::Created => result.created += 1,
                UpsertAction::Updated => result.updated += 1,
                UpsertAction::Skipped => result.skipped += 1,
            },
            Err(e) => log::warn!("Failed to sync {}: {}", file_path, e),
        }
    }

    if delete_orphans {
        let db_nodes = node_repo::list_by_vault(&pool, vault_id)
            .await
            .map_err(|e| e.to_string())?;

        for node in db_nodes {
            if let Some(ref path) = node.file_path {
                if !seen_paths.contains(path) {
                    if vault_sync::delete_by_path(&pool, &graph, vault_id, path)
                        .await
                        .map_err(|e| e.to_string())?
                    {
                        result.deleted += 1;
                    }
                }
            }
        }
    }

    Ok(result)
}

#[tauri::command]
pub async fn get_random_node(
    state: tauri::State<'_, AppState>,
    vault_id: Uuid,
) -> Result<Node, String> {
    dispatch(
        &state,
        "get_random_node",
        serde_json::to_value(vault_id).map_err(|e| e.to_string())?,
    )
    .await
}
