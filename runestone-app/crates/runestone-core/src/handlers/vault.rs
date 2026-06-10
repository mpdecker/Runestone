use crate::context::BackendContext;
use crate::db::{run_neo4j_init, run_pg_migrations};
use crate::models::vault::{CreateVaultRequest, Vault};

pub async fn init_database(ctx: &BackendContext) -> Result<String, String> {
    run_pg_migrations(&ctx.pg)
        .await
        .map_err(|e| format!("PostgreSQL migration failed: {}", e))?;

    run_neo4j_init(&ctx.neo4j)
        .await
        .map_err(|e| format!("Neo4j initialization failed: {}", e))?;

    Ok("Database initialized successfully".to_string())
}

pub async fn create_vault(
    ctx: &BackendContext,
    request: CreateVaultRequest,
) -> Result<Vault, String> {
    let row = sqlx::query_as::<_, Vault>(
        "INSERT INTO vaults (name, root_path) VALUES ($1, $2) RETURNING id, name, root_path, created_at, updated_at",
    )
    .bind(&request.name)
    .bind(&request.root_path)
    .fetch_one(&ctx.pg)
    .await
    .map_err(|e| format!("Failed to create vault: {}", e))?;

    Ok(row)
}

pub async fn list_vaults(ctx: &BackendContext) -> Result<Vec<Vault>, String> {
    let vaults = sqlx::query_as::<_, Vault>(
        "SELECT id, name, root_path, created_at, updated_at FROM vaults ORDER BY name",
    )
    .fetch_all(&ctx.pg)
    .await
    .map_err(|e| format!("Failed to list vaults: {}", e))?;

    Ok(vaults)
}
