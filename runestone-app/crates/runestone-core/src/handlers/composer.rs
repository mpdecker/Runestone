use crate::context::BackendContext;
use crate::models::node::Node;
use crate::repositories::node_repo;
use crate::services::graph_sync;
use uuid::Uuid;

pub async fn merge_nodes(
    ctx: &BackendContext,
    source_id: Uuid,
    target_id: Uuid,
) -> Result<Node, String> {
    let source = node_repo::get_by_id(&ctx.pg, source_id)
        .await
        .map_err(|e| e.to_string())?;

    let target = node_repo::get_by_id(&ctx.pg, target_id)
        .await
        .map_err(|e| e.to_string())?;

    let merged_content = format!(
        "{}\n\n<hr>\n<h2>{}</h2>\n{}",
        target.content, source.title, source.content
    );
    let word_count = merged_content.split_whitespace().count() as i32;

    let updated = sqlx::query_as::<_, Node>(
        "UPDATE nodes SET content = $2, word_count = $3, updated_at = NOW() WHERE id = $1 RETURNING id, vault_id, title, content, content_type, file_path, metadata, word_count, created_at, updated_at",
    )
    .bind(target_id)
    .bind(&merged_content)
    .bind(word_count)
    .fetch_one(&ctx.pg)
    .await
    .map_err(|e| format!("Failed to merge: {}", e))?;

    graph_sync::update_node(&ctx.neo4j, target_id, &updated.title, &updated.content_type)
        .await
        .map_err(|e| format!("Neo4j update failed during merge: {}", e))?;

    graph_sync::delete_node_before_pg(&ctx.neo4j, source_id)
        .await
        .map_err(|e| format!("Neo4j delete failed during merge: {}", e))?;

    node_repo::delete_by_id(&ctx.pg, source_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(updated)
}

pub async fn split_node(
    ctx: &BackendContext,
    node_id: Uuid,
    new_title: String,
) -> Result<(Node, Node), String> {
    let source = node_repo::get_by_id(&ctx.pg, node_id)
        .await
        .map_err(|e| e.to_string())?;

    let split_point = source.content.len() / 2;
    let first_content = source.content[..split_point].to_string();
    let second_content = source.content[split_point..].to_string();

    let first_wc = first_content.split_whitespace().count() as i32;
    let second_wc = second_content.split_whitespace().count() as i32;

    let first = sqlx::query_as::<_, Node>(
        "UPDATE nodes SET content = $2, word_count = $3, updated_at = NOW() WHERE id = $1 RETURNING id, vault_id, title, content, content_type, file_path, metadata, word_count, created_at, updated_at",
    )
    .bind(node_id)
    .bind(&first_content)
    .bind(first_wc)
    .fetch_one(&ctx.pg)
    .await
    .map_err(|e| format!("Failed to update source: {}", e))?;

    let new_id = Uuid::new_v4();
    let second = sqlx::query_as::<_, Node>(
        "INSERT INTO nodes (id, vault_id, title, content, content_type, word_count) VALUES ($1, $2, $3, $4, 'note', $5) RETURNING id, vault_id, title, content, content_type, file_path, metadata, word_count, created_at, updated_at",
    )
    .bind(new_id)
    .bind(source.vault_id)
    .bind(&new_title)
    .bind(&second_content)
    .bind(second_wc)
    .fetch_one(&ctx.pg)
    .await
    .map_err(|e| format!("Failed to create split node: {}", e))?;

    graph_sync::create_node_with_pg_rollback(
        &ctx.neo4j,
        &ctx.pg,
        new_id,
        source.vault_id,
        &second.title,
        &second.content_type,
    )
    .await
    .map_err(|e| e.to_string())?;

    graph_sync::update_node(&ctx.neo4j, node_id, &first.title, &first.content_type)
        .await
        .map_err(|e| format!("Neo4j update failed on split source: {}", e))?;

    graph_sync::create_relates_to(&ctx.neo4j, node_id, new_id)
        .await
        .map_err(|e| format!("Neo4j relation failed: {}", e))?;

    Ok((first, second))
}
