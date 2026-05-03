use crate::models::node::Node;
use crate::state::AppState;
use uuid::Uuid;

#[tauri::command]
pub async fn merge_nodes(
    state: tauri::State<'_, AppState>,
    source_id: Uuid,
    target_id: Uuid,
) -> Result<Node, String> {
    let source = sqlx::query_as::<_, Node>(
        "SELECT id, vault_id, title, content, content_type, file_path, metadata, word_count, created_at, updated_at FROM nodes WHERE id = $1",
    )
    .bind(source_id)
    .fetch_one(state.pg()?)
    .await
    .map_err(|e| format!("Source node not found: {}", e))?;

    let mut target = sqlx::query_as::<_, Node>(
        "SELECT id, vault_id, title, content, content_type, file_path, metadata, word_count, created_at, updated_at FROM nodes WHERE id = $1",
    )
    .bind(target_id)
    .fetch_one(state.pg()?)
    .await
    .map_err(|e| format!("Target node not found: {}", e))?;

    let merged_content = format!("{}\n\n<hr>\n<h2>{}</h2>\n{}", target.content, source.title, source.content);
    let word_count = merged_content.split_whitespace().count() as i32;

    let updated = sqlx::query_as::<_, Node>(
        "UPDATE nodes SET content = $2, word_count = $3, updated_at = NOW() WHERE id = $1 RETURNING id, vault_id, title, content, content_type, file_path, metadata, word_count, created_at, updated_at",
    )
    .bind(target_id)
    .bind(&merged_content)
    .bind(word_count)
    .fetch_one(state.pg()?)
    .await
    .map_err(|e| format!("Failed to merge: {}", e))?;

    sqlx::query("DELETE FROM nodes WHERE id = $1")
        .bind(source_id)
        .execute(state.pg()?)
        .await
        .map_err(|e| format!("Failed to delete source: {}", e))?;

    let _ = state.neo4j()?.run(neo4rs::query("MATCH (n:Node {pg_id: $pg_id}) DETACH DELETE n")
        .param("pg_id", source_id.to_string())).await;

    Ok(updated)
}

#[tauri::command]
pub async fn split_node(
    state: tauri::State<'_, AppState>,
    node_id: Uuid,
    new_title: String,
) -> Result<(Node, Node), String> {
    let source = sqlx::query_as::<_, Node>(
        "SELECT id, vault_id, title, content, content_type, file_path, metadata, word_count, created_at, updated_at FROM nodes WHERE id = $1",
    )
    .bind(node_id)
    .fetch_one(state.pg()?)
    .await
    .map_err(|e| format!("Node not found: {}", e))?;

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
    .fetch_one(state.pg()?)
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
    .fetch_one(state.pg()?)
    .await
    .map_err(|e| format!("Failed to create split node: {}", e))?;

    let _ = state.neo4j()?.run(neo4rs::query("CREATE (n:Node {pg_id: $pg_id, vault_id: $vault_id, title: $title, content_type: 'note'})")
        .param("pg_id", new_id.to_string())
        .param("vault_id", source.vault_id.to_string())
        .param("title", second.title.clone())).await;

    let _ = state.neo4j()?.run(neo4rs::query("MATCH (a:Node {pg_id: $a}), (b:Node {pg_id: $b}) CREATE (a)-[:RELATES_TO]->(b)")
        .param("a", node_id.to_string())
        .param("b", new_id.to_string())).await;

    Ok((first, second))
}
