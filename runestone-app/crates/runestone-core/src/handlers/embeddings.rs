use crate::context::BackendContext;
use crate::embedding::generate_embedding;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingStatus {
    pub pending: i64,
    pub processing: i64,
    pub completed: i64,
    pub failed: i64,
}

pub async fn enqueue_embedding(ctx: &BackendContext, node_id: Uuid) -> Result<(), String> {
    sqlx::query(
        r#"INSERT INTO embedding_jobs (node_id, status, attempts, last_error, updated_at)
           VALUES ($1, 'pending', 0, NULL, NOW())
           ON CONFLICT (node_id) DO UPDATE
           SET status = 'pending', attempts = 0, updated_at = NOW()
           WHERE embedding_jobs.status IN ('failed', 'completed', 'processing')"#,
    )
    .bind(node_id)
    .execute(&ctx.pg)
    .await
    .map_err(|e| format!("Failed to enqueue embedding: {}", e))?;

    Ok(())
}

/// Jobs left in 'processing' by a crash/restart would otherwise never run again.
const STALE_PROCESSING_MINUTES: i32 = 5;
/// Failed jobs are retried (Ollama not started yet, transient errors) with a growing delay.
const MAX_ATTEMPTS: i32 = 5;

pub async fn process_pending_jobs(ctx: &BackendContext, batch_size: i64) -> Result<u64, String> {
    sqlx::query(
        "UPDATE embedding_jobs SET status = 'pending', updated_at = NOW() WHERE status = 'processing' AND updated_at < NOW() - make_interval(mins => $1)",
    )
    .bind(STALE_PROCESSING_MINUTES)
    .execute(&ctx.pg)
    .await
    .map_err(|e| format!("Failed to reset stale jobs: {}", e))?;

    let jobs = sqlx::query_as::<_, (Uuid, Uuid)>(
        r#"UPDATE embedding_jobs
           SET status = 'processing', updated_at = NOW()
           WHERE id IN (
               SELECT id FROM embedding_jobs
               WHERE status = 'pending'
                  OR (status = 'failed' AND attempts < $2
                      AND updated_at < NOW() - make_interval(mins => attempts))
               ORDER BY created_at
               LIMIT $1
               FOR UPDATE SKIP LOCKED
           )
           RETURNING id, node_id"#,
    )
    .bind(batch_size)
    .bind(MAX_ATTEMPTS)
    .fetch_all(&ctx.pg)
    .await
    .map_err(|e| format!("Failed to fetch jobs: {}", e))?;

    let mut processed = 0u64;
    for (job_id, node_id) in jobs {
        let row =
            sqlx::query_as::<_, (String, String)>("SELECT title, content FROM nodes WHERE id = $1")
                .bind(node_id)
                .fetch_optional(&ctx.pg)
                .await
                .map_err(|e| format!("Node lookup failed: {}", e))?;

        let Some((title, content)) = row else {
            mark_job_failed(&ctx.pg, job_id, "Node not found").await?;
            continue;
        };

        if content.is_empty() {
            mark_job_completed(&ctx.pg, job_id).await?;
            processed += 1;
            continue;
        }

        // Notes are HTML; embed the text, and keep within typical embedding-model context limits.
        let plain = crate::util::plain_text(&content);
        let embed_text = format!("{}: {}", title, crate::util::truncate_chars(&plain, 6000));
        match generate_embedding(&embed_text, &ctx.embed_config).await {
            Ok(embedding) => {
                let vector = pgvector::Vector::from(embedding);
                match sqlx::query("UPDATE nodes SET embedding = $1 WHERE id = $2")
                    .bind(vector)
                    .bind(node_id)
                    .execute(&ctx.pg)
                    .await
                {
                    Ok(_) => {
                        mark_job_completed(&ctx.pg, job_id).await?;
                        processed += 1;
                    }
                    Err(e) => {
                        mark_job_failed(
                            &ctx.pg,
                            job_id,
                            &format!("Failed to store embedding: {}", e),
                        )
                        .await?;
                    }
                }
            }
            Err(e) => {
                mark_job_failed(&ctx.pg, job_id, &e).await?;
            }
        }
    }

    Ok(processed)
}

async fn mark_job_completed(pool: &sqlx::PgPool, job_id: Uuid) -> Result<(), String> {
    // Only a job that is still 'processing' may complete: if the note was edited meanwhile,
    // enqueue_embedding put it back to 'pending' and that must not be overwritten.
    sqlx::query("UPDATE embedding_jobs SET status = 'completed', updated_at = NOW() WHERE id = $1 AND status = 'processing'")
        .bind(job_id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

async fn mark_job_failed(pool: &sqlx::PgPool, job_id: Uuid, error: &str) -> Result<(), String> {
    sqlx::query(
        "UPDATE embedding_jobs SET status = 'failed', attempts = attempts + 1, last_error = $2, updated_at = NOW() WHERE id = $1 AND status = 'processing'",
    )
    .bind(job_id)
    .bind(error)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn reindex_vault(ctx: &BackendContext, vault_id: Uuid) -> Result<u64, String> {
    let node_ids = sqlx::query_as::<_, (Uuid,)>("SELECT id FROM nodes WHERE vault_id = $1")
        .bind(vault_id)
        .fetch_all(&ctx.pg)
        .await
        .map_err(|e| format!("Failed to list nodes: {}", e))?;

    let mut count = 0u64;
    for (node_id,) in node_ids {
        enqueue_embedding(ctx, node_id).await?;
        count += 1;
    }
    Ok(count)
}

pub async fn get_embedding_status(ctx: &BackendContext) -> Result<EmbeddingStatus, String> {
    let rows = sqlx::query_as::<_, (String, i64)>(
        "SELECT status, COUNT(*) FROM embedding_jobs GROUP BY status",
    )
    .fetch_all(&ctx.pg)
    .await
    .map_err(|e| format!("Failed to get status: {}", e))?;

    let mut status = EmbeddingStatus {
        pending: 0,
        processing: 0,
        completed: 0,
        failed: 0,
    };

    for (s, count) in rows {
        match s.as_str() {
            "pending" => status.pending = count,
            "processing" => status.processing = count,
            "completed" => status.completed = count,
            "failed" => status.failed = count,
            _ => {}
        }
    }

    Ok(status)
}

pub fn spawn_embedding_worker(ctx: BackendContext) {
    tokio::spawn(async move {
        loop {
            match process_pending_jobs(&ctx, 5).await {
                Ok(n) if n > 0 => log::info!("Processed {} embedding jobs", n),
                Ok(_) => {}
                Err(e) => log::warn!("Embedding worker error: {}", e),
            }
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        }
    });
}
