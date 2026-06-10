use crate::router::dispatch;
use crate::document::{chunk_text, parse_document};
use crate::embedding::generate_embedding;
use crate::models::extraction::ExtractionNode;
use crate::models::node::Node;
use crate::path_guard::canonicalize_path;
use crate::services::graph_sync;
use crate::state::AppState;
use uuid::Uuid;

#[tauri::command]
pub async fn import_document(
    state: tauri::State<'_, AppState>,
    vault_id: Uuid,
    file_path: String,
) -> Result<Node, String> {
    dispatch(
        &state,
        "import_document",
        serde_json::json!({ "vault_id": vault_id, "file_path": file_path }),
    )
    .await
}

pub async fn import_document_impl(
    state: &AppState,
    vault_id: Uuid,
    file_path: String,
) -> Result<Node, String> {
    let safe_path = canonicalize_path(&file_path).map_err(|e| e.to_string())?;
    let file_path = safe_path.to_string_lossy().to_string();
    let text = parse_document(&file_path)?;

    let title = std::path::Path::new(&file_path)
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let id = Uuid::new_v4();
    let wc = text.split_whitespace().count() as i32;

    let node = sqlx::query_as::<_, Node>(
        "INSERT INTO nodes (id, vault_id, title, content, content_type, file_path, word_count) VALUES ($1, $2, $3, $4, 'document', $5, $6) RETURNING id, vault_id, title, content, content_type, file_path, metadata, word_count, created_at, updated_at",
    )
    .bind(id)
    .bind(vault_id)
    .bind(&title)
    .bind(&text)
    .bind(&file_path)
    .bind(wc)
    .fetch_one(&state.pg()?)
    .await
    .map_err(|e| format!("Failed to insert document node: {}", e))?;

    graph_sync::create_node_with_pg_rollback(
        &state.neo4j()?,
        &state.pg()?,
        id,
        vault_id,
        &node.title,
        "document",
    )
    .await
    .map_err(|e| e.to_string())?;

    let chunks = chunk_text(&text, 1000, 50);
    for (i, chunk) in chunks.iter().enumerate() {
        let chunk_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO document_chunks (id, document_node_id, chunk_index, content, token_count) VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(chunk_id)
        .bind(id)
        .bind(i as i32)
        .bind(chunk)
        .bind(chunk.split_whitespace().count() as i32)
        .execute(&state.pg()?)
        .await
        .map_err(|e| format!("Failed to insert document chunk: {}", e))?;
    }

    let pg = state.pg()?;
    let config = state.embed_config.clone();
    let embed_text = format!("{}: {}", node.title, node.content);
    let doc_id = id;
    tokio::spawn(async move {
        if let Ok(embedding) = generate_embedding(&embed_text, &config).await {
            let vector = pgvector::Vector::from(embedding);
            let _ = sqlx::query("UPDATE nodes SET embedding = $1 WHERE id = $2")
                .bind(vector)
                .bind(doc_id)
                .execute(&pg)
                .await;
        }
    });

    Ok(node)
}

#[tauri::command]
pub async fn extract_from_document(
    state: tauri::State<'_, AppState>,
    node_id: Uuid,
) -> Result<Vec<ExtractionNode>, String> {
    dispatch(
        &state,
        "extract_from_document",
        serde_json::to_value(node_id).map_err(|e| e.to_string())?,
    )
    .await
}
