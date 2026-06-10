use crate::context::BackendContext;
use crate::models::extraction::PendingExtraction;
use crate::repositories::node_repo;
use crate::services::graph_sync;
use uuid::Uuid;

pub async fn get_pending_extractions(
    ctx: &BackendContext,
    vault_id: Uuid,
) -> Result<Vec<PendingExtraction>, String> {
    let extractions = sqlx::query_as::<_, PendingExtraction>(
        r#"SELECT id, title, content_type, metadata, created_at
           FROM nodes
           WHERE vault_id = $1
             AND (content_type = 'entity' OR content_type = 'concept')
             AND metadata->>'status' = 'pending'
           ORDER BY created_at DESC"#,
    )
    .bind(vault_id)
    .fetch_all(&ctx.pg)
    .await
    .map_err(|e| format!("Failed to get pending extractions: {}", e))?;

    Ok(extractions)
}

pub async fn approve_extraction(ctx: &BackendContext, extraction_id: Uuid) -> Result<(), String> {
    let metadata = serde_json::json!({"status": "approved"});

    sqlx::query("UPDATE nodes SET metadata = metadata || $1 WHERE id = $2")
        .bind(&metadata)
        .bind(extraction_id)
        .execute(&ctx.pg)
        .await
        .map_err(|e| format!("Failed to approve extraction: {}", e))?;

    Ok(())
}

pub async fn reject_extraction(ctx: &BackendContext, extraction_id: Uuid) -> Result<(), String> {
    graph_sync::delete_node_before_pg(&ctx.neo4j, extraction_id)
        .await
        .map_err(|e| format!("Neo4j delete failed: {}", e))?;

    node_repo::delete_by_id(&ctx.pg, extraction_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

pub async fn batch_approve_extractions(
    ctx: &BackendContext,
    extraction_ids: Vec<Uuid>,
) -> Result<(), String> {
    let metadata = serde_json::json!({"status": "approved"});

    for id in &extraction_ids {
        sqlx::query("UPDATE nodes SET metadata = metadata || $1 WHERE id = $2")
            .bind(&metadata)
            .bind(id)
            .execute(&ctx.pg)
            .await
            .map_err(|e| format!("Failed to approve extraction {}: {}", id, e))?;
    }

    Ok(())
}
