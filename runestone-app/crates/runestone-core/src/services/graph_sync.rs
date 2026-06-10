use crate::error::AppResult;
use std::sync::Arc;
use uuid::Uuid;

pub async fn create_node(
    graph: &Arc<neo4rs::Graph>,
    pg_id: Uuid,
    vault_id: Uuid,
    title: &str,
    content_type: &str,
) -> AppResult<()> {
    graph
        .run(
            neo4rs::query(
                "CREATE (n:Node {pg_id: $pg_id, vault_id: $vault_id, title: $title, content_type: $content_type})",
            )
            .param("pg_id", pg_id.to_string())
            .param("vault_id", vault_id.to_string())
            .param("title", title)
            .param("content_type", content_type),
        )
        .await?;
    Ok(())
}

pub async fn update_node(
    graph: &Arc<neo4rs::Graph>,
    pg_id: Uuid,
    title: &str,
    content_type: &str,
) -> AppResult<()> {
    graph
        .run(
            neo4rs::query(
                "MATCH (n:Node {pg_id: $pg_id}) SET n.title = $title, n.content_type = $content_type",
            )
            .param("pg_id", pg_id.to_string())
            .param("title", title)
            .param("content_type", content_type),
        )
        .await?;
    Ok(())
}

pub async fn delete_node(graph: &Arc<neo4rs::Graph>, pg_id: Uuid) -> AppResult<()> {
    graph
        .run(
            neo4rs::query("MATCH (n:Node {pg_id: $pg_id}) DETACH DELETE n")
                .param("pg_id", pg_id.to_string()),
        )
        .await?;
    Ok(())
}

pub async fn add_tag(
    graph: &Arc<neo4rs::Graph>,
    pg_id: Uuid,
    tag_name: &str,
) -> AppResult<()> {
    graph
        .run(
            neo4rs::query(
                "MERGE (t:Tag {name: $tag_name}) WITH t MATCH (n:Node {pg_id: $pg_id}) MERGE (n)-[:HAS_TAG]->(t)",
            )
            .param("tag_name", tag_name)
            .param("pg_id", pg_id.to_string()),
        )
        .await?;
    Ok(())
}

pub async fn remove_tag(
    graph: &Arc<neo4rs::Graph>,
    pg_id: Uuid,
    tag_name: &str,
) -> AppResult<()> {
    graph
        .run(
            neo4rs::query(
                "MATCH (n:Node {pg_id: $pg_id})-[r:HAS_TAG]->(t:Tag {name: $tag_name}) DELETE r",
            )
            .param("pg_id", pg_id.to_string())
            .param("tag_name", tag_name),
        )
        .await?;
    Ok(())
}

pub async fn create_wiki_link(
    graph: &Arc<neo4rs::Graph>,
    source_id: Uuid,
    target_id: Uuid,
) -> AppResult<()> {
    graph
        .run(
            neo4rs::query(
                "MATCH (a:Node {pg_id: $a_id}), (b:Node {pg_id: $b_id}) CREATE (a)-[:LINKS_TO {context: 'wiki-link'}]->(b)",
            )
            .param("a_id", source_id.to_string())
            .param("b_id", target_id.to_string()),
        )
        .await?;
    Ok(())
}

pub async fn create_relates_to(
    graph: &Arc<neo4rs::Graph>,
    from_id: Uuid,
    to_id: Uuid,
) -> AppResult<()> {
    graph
        .run(
            neo4rs::query(
                "MATCH (a:Node {pg_id: $a}), (b:Node {pg_id: $b}) CREATE (a)-[:RELATES_TO]->(b)",
            )
            .param("a", from_id.to_string())
            .param("b", to_id.to_string()),
        )
        .await?;
    Ok(())
}

pub async fn create_extracted_entity(
    graph: &Arc<neo4rs::Graph>,
    pg_id: Uuid,
    doc_id: Uuid,
    title: &str,
    confidence: f64,
    chunk_index: i32,
) -> AppResult<()> {
    graph
        .run(
            neo4rs::query(
                "MATCH (doc:Node {pg_id: $doc_id}) CREATE (n:Node {pg_id: $pg_id, vault_id: doc.vault_id, title: $title, content_type: 'entity'}) CREATE (n)-[:EXTRACTED_FROM {confidence: $conf, chunk_index: $chunk}]->(doc)",
            )
            .param("doc_id", doc_id.to_string())
            .param("pg_id", pg_id.to_string())
            .param("title", title)
            .param("conf", confidence)
            .param("chunk", chunk_index),
        )
        .await?;
    Ok(())
}

pub async fn create_extracted_concept(
    graph: &Arc<neo4rs::Graph>,
    pg_id: Uuid,
    doc_id: Uuid,
    title: &str,
    confidence: f64,
    chunk_index: i32,
) -> AppResult<()> {
    graph
        .run(
            neo4rs::query(
                "MATCH (doc:Node {pg_id: $doc_id}) CREATE (n:Node {pg_id: $pg_id, vault_id: doc.vault_id, title: $title, content_type: 'concept'}) CREATE (n)-[:EXTRACTED_FROM {confidence: $conf, chunk_index: $chunk}]->(doc)",
            )
            .param("doc_id", doc_id.to_string())
            .param("pg_id", pg_id.to_string())
            .param("title", title)
            .param("conf", confidence)
            .param("chunk", chunk_index),
        )
        .await?;
    Ok(())
}

pub async fn create_extraction_edge(
    graph: &Arc<neo4rs::Graph>,
    from_id: Uuid,
    to_id: Uuid,
    relation: &str,
    confidence: f64,
) -> AppResult<()> {
    graph
        .run(
            neo4rs::query(
                "MATCH (a:Node {pg_id: $from_id}), (b:Node {pg_id: $to_id}) CREATE (a)-[:RELATES_TO {relation: $relation, confidence: $conf}]->(b)",
            )
            .param("from_id", from_id.to_string())
            .param("to_id", to_id.to_string())
            .param("relation", relation)
            .param("conf", confidence),
        )
        .await?;
    Ok(())
}

/// Delete Neo4j node first; on failure, caller should not delete from PostgreSQL.
pub async fn delete_node_before_pg(
    graph: &Arc<neo4rs::Graph>,
    pg_id: Uuid,
) -> AppResult<()> {
    delete_node(graph, pg_id).await
}

/// Create in Neo4j after PG insert; on failure, roll back PG row.
pub async fn create_node_with_pg_rollback(
    graph: &Arc<neo4rs::Graph>,
    pool: &sqlx::PgPool,
    pg_id: Uuid,
    vault_id: Uuid,
    title: &str,
    content_type: &str,
) -> AppResult<()> {
    if let Err(e) = create_node(graph, pg_id, vault_id, title, content_type).await {
        if let Err(del_err) = sqlx::query("DELETE FROM nodes WHERE id = $1")
            .bind(pg_id)
            .execute(pool)
            .await
        {
            log::error!(
                "Neo4j create failed and PG rollback failed for node {}: {}",
                pg_id,
                del_err
            );
        }
        return Err(e);
    }
    Ok(())
}
