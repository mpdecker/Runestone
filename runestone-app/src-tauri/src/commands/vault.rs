use crate::db::{run_neo4j_init, run_pg_migrations};
use crate::models::vault::{CreateVaultRequest, Vault};
use crate::state::AppState;

#[tauri::command]
pub async fn init_database(state: tauri::State<'_, AppState>) -> Result<String, String> {
    run_pg_migrations(state.pg()?)
        .await
        .map_err(|e| format!("PostgreSQL migration failed: {}", e))?;

    run_neo4j_init(state.neo4j()?)
        .await
        .map_err(|e| format!("Neo4j initialization failed: {}", e))?;

    Ok("Database initialized successfully".to_string())
}

#[tauri::command]
pub async fn create_vault(
    state: tauri::State<'_, AppState>,
    request: CreateVaultRequest,
) -> Result<Vault, String> {
    let row = sqlx::query_as::<_, Vault>(
        "INSERT INTO vaults (name, root_path) VALUES ($1, $2) RETURNING id, name, root_path, created_at, updated_at",
    )
    .bind(&request.name)
    .bind(&request.root_path)
    .fetch_one(state.pg()?)
    .await
    .map_err(|e| format!("Failed to create vault: {}", e))?;

    Ok(row)
}

#[tauri::command]
pub async fn list_vaults(state: tauri::State<'_, AppState>) -> Result<Vec<Vault>, String> {
    let vaults = sqlx::query_as::<_, Vault>(
        "SELECT id, name, root_path, created_at, updated_at FROM vaults ORDER BY name",
    )
    .fetch_all(state.pg()?)
    .await
    .map_err(|e| format!("Failed to list vaults: {}", e))?;

    Ok(vaults)
}
