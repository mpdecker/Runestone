use crate::context::BackendContext;
use crate::llm::extract_from_text;
use crate::models::extraction::ExtractionNode;
use crate::repositories::node_repo;
use crate::services::graph_sync;
use uuid::Uuid;

pub async fn extract_from_document(
    ctx: &BackendContext,
    node_id: Uuid,
) -> Result<Vec<ExtractionNode>, String> {
    let chunk_rows = sqlx::query_as::<_, (Uuid, i32, String)>(
        "SELECT id, chunk_index, content FROM document_chunks WHERE document_node_id = $1 ORDER BY chunk_index",
    )
    .bind(node_id)
    .fetch_all(&ctx.pg)
    .await
    .map_err(|e| format!("Failed to get document chunks: {}", e))?;

    if chunk_rows.is_empty() {
        return Err("No chunks found. Import the document first.".to_string());
    }

    let doc_node = node_repo::get_by_id(&ctx.pg, node_id)
        .await
        .map_err(|e| e.to_string())?;
    let vault_id = doc_node.vault_id;

    let mut all_extractions: Vec<ExtractionNode> = Vec::new();
    for (_chunk_id, chunk_index, chunk_content) in &chunk_rows {
        let extraction = match extract_from_text(chunk_content, &ctx.llm_config).await {
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

            sqlx::query(
                "INSERT INTO nodes (id, vault_id, title, content, content_type, metadata) SELECT $1, vault_id, $2, $3, 'entity', $4 FROM nodes WHERE id = $5",
            )
            .bind(id)
            .bind(&entity.name)
            .bind(&entity.description)
            .bind(&metadata)
            .bind(node_id)
            .execute(&ctx.pg)
            .await
            .map_err(|e| format!("Failed to insert extracted entity: {}", e))?;

            graph_sync::create_extracted_entity(
                &ctx.neo4j,
                id,
                node_id,
                &entity.name,
                confidence,
                *chunk_index,
            )
            .await
            .map_err(|e| e.to_string())?;

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

            sqlx::query(
                "INSERT INTO nodes (id, vault_id, title, content, content_type, metadata) SELECT $1, vault_id, $2, $3, 'concept', $4 FROM nodes WHERE id = $5",
            )
            .bind(id)
            .bind(&concept.name)
            .bind(&concept.description)
            .bind(&metadata)
            .bind(node_id)
            .execute(&ctx.pg)
            .await
            .map_err(|e| format!("Failed to insert extracted concept: {}", e))?;

            graph_sync::create_extracted_concept(
                &ctx.neo4j,
                id,
                node_id,
                &concept.name,
                confidence,
                *chunk_index,
            )
            .await
            .map_err(|e| e.to_string())?;

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
            let source = node_repo::find_by_title(&ctx.pg, vault_id, &rel.source)
                .await
                .map_err(|e| e.to_string())?;
            let target = node_repo::find_by_title(&ctx.pg, vault_id, &rel.target)
                .await
                .map_err(|e| e.to_string())?;

            if let (Some(source), Some(target)) = (source, target) {
                graph_sync::create_extraction_edge(
                    &ctx.neo4j,
                    source.id,
                    target.id,
                    &rel.rel_type,
                    confidence,
                )
                .await
                .map_err(|e| e.to_string())?;
            }
        }
    }

    Ok(all_extractions)
}
