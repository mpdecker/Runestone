use crate::document::{chunk_text, parse_document};
use crate::embedding::generate_embedding;
use crate::llm::extract_from_text;
use crate::models::extraction::ExtractionNode;
use crate::models::node::Node;
use crate::state::AppState;
use uuid::Uuid;

#[tauri::command]
pub async fn import_document(
    state: tauri::State<'_, AppState>,
    vault_id: Uuid,
    file_path: String,
) -> Result<Node, String> {
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
    .fetch_one(state.pg()?)
    .await
    .map_err(|e| format!("Failed to insert document node: {}", e))?;

    let _ = state
        .neo4j()?
        .run(
            neo4rs::query("CREATE (n:Node {pg_id: $pg_id, vault_id: $vault_id, title: $title, content_type: 'document'})")
                .param("pg_id", id.to_string())
                .param("vault_id", vault_id.to_string())
                .param("title", node.title.clone()),
        )
        .await;

    let chunks = chunk_text(&text, 1000, 50);
    for (i, chunk) in chunks.iter().enumerate() {
        let chunk_id = Uuid::new_v4();
        let _ = sqlx::query(
            "INSERT INTO document_chunks (id, document_node_id, chunk_index, content, token_count) VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(chunk_id)
        .bind(id)
        .bind(i as i32)
        .bind(chunk)
        .bind(chunk.split_whitespace().count() as i32)
        .execute(state.pg()?)
        .await;
    }

    let pg = state.pg()?.clone();
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
    let chunk_rows = sqlx::query_as::<_, (Uuid, i32, String)>(
        "SELECT id, chunk_index, content FROM document_chunks WHERE document_node_id = $1 ORDER BY chunk_index",
    )
    .bind(node_id)
    .fetch_all(state.pg()?)
    .await
    .map_err(|e| format!("Failed to get document chunks: {}", e))?;

    if chunk_rows.is_empty() {
        return Err("No chunks found. Import the document first.".to_string());
    }

    let mut all_extractions: Vec<ExtractionNode> = Vec::new();

    for (_chunk_id, chunk_index, chunk_content) in &chunk_rows {
        let extraction = match extract_from_text(chunk_content, &state.llm_config).await {
            Ok(ext) => ext,
            Err(e) => {
                log::warn!("LLM extraction failed for chunk {}: {}", chunk_index, e);
                continue;
            }
        };

        let confidence = 0.8;

        for entity in &extraction.entities {
            let id = Uuid::new_v4();
            let metadata = serde_json::json!({
                "status": "pending",
                "confidence": confidence,
                "source_chunk": chunk_index,
                "description": entity.description,
                "extraction_type": entity.entity_type,
            });

            let _ = sqlx::query(
                "INSERT INTO nodes (id, vault_id, title, content, content_type, metadata) SELECT $1, vault_id, $2, $3, 'entity', $4 FROM nodes WHERE id = $5",
            )
            .bind(id)
            .bind(&entity.name)
            .bind(&entity.description)
            .bind(&metadata)
            .bind(node_id)
            .execute(state.pg()?)
            .await;

            let _ = state.neo4j()?.run(
                neo4rs::query("MATCH (doc:Node {pg_id: $doc_id}) CREATE (n:Node {pg_id: $pg_id, vault_id: doc.vault_id, title: $title, content_type: 'entity'}) CREATE (n)-[:EXTRACTED_FROM {confidence: $conf, chunk_index: $chunk}]->(doc)")
                    .param("doc_id", node_id.to_string())
                    .param("pg_id", id.to_string())
                    .param("title", entity.name.clone())
                    .param("conf", confidence)
                    .param("chunk", *chunk_index),
            ).await;

            all_extractions.push(ExtractionNode {
                name: entity.name.clone(),
                description: entity.description.clone(),
                extraction_type: "entity".to_string(),
                confidence,
                source_node_id: node_id,
                chunk_index: *chunk_index,
            });
        }

        for concept in &extraction.concepts {
            let id = Uuid::new_v4();
            let metadata = serde_json::json!({
                "status": "pending",
                "confidence": confidence,
                "source_chunk": chunk_index,
                "description": concept.description,
                "extraction_type": "concept",
            });

            let _ = sqlx::query(
                "INSERT INTO nodes (id, vault_id, title, content, content_type, metadata) SELECT $1, vault_id, $2, $3, 'concept', $4 FROM nodes WHERE id = $5",
            )
            .bind(id)
            .bind(&concept.name)
            .bind(&concept.description)
            .bind(&metadata)
            .bind(node_id)
            .execute(state.pg()?)
            .await;

            let _ = state.neo4j()?.run(
                neo4rs::query("MATCH (doc:Node {pg_id: $doc_id}) CREATE (n:Node {pg_id: $pg_id, vault_id: doc.vault_id, title: $title, content_type: 'concept'}) CREATE (n)-[:EXTRACTED_FROM {confidence: $conf, chunk_index: $chunk}]->(doc)")
                    .param("doc_id", node_id.to_string())
                    .param("pg_id", id.to_string())
                    .param("title", concept.name.clone())
                    .param("conf", confidence)
                    .param("chunk", *chunk_index),
            ).await;

            all_extractions.push(ExtractionNode {
                name: concept.name.clone(),
                description: concept.description.clone(),
                extraction_type: "concept".to_string(),
                confidence,
                source_node_id: node_id,
                chunk_index: *chunk_index,
            });
        }

        for rel in &extraction.relationships {
            let _ = state.neo4j()?.run(
                neo4rs::query("MATCH (a:Node {title: $source}), (b:Node {title: $target}) CREATE (a)-[:RELATES_TO {description: $desc, type: $rel_type, confidence: $conf}]->(b)")
                    .param("source", rel.source.clone())
                    .param("target", rel.target.clone())
                    .param("desc", rel.description.clone())
                    .param("rel_type", rel.rel_type.clone())
                    .param("conf", confidence),
            ).await;
        }
    }

    Ok(all_extractions)
}
