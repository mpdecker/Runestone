use crate::models::node::{CreateNodeRequest, Node, NodeListItem};
use crate::state::AppState;
use uuid::Uuid;

#[tauri::command]
pub async fn create_daily_note(
    state: tauri::State<'_, AppState>,
    vault_id: Uuid,
) -> Result<Node, String> {
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();

    let existing = sqlx::query_as::<_, Node>(
        "SELECT id, vault_id, title, content, content_type, file_path, metadata, word_count, created_at, updated_at FROM nodes WHERE vault_id = $1 AND title = $2 LIMIT 1",
    )
    .bind(vault_id)
    .bind(&today)
    .fetch_optional(state.pg()?)
    .await
    .map_err(|e| format!("Query error: {}", e))?;

    if let Some(node) = existing {
        return Ok(node);
    }

    let content = format!(
        "<h1>{}</h1><p></p><h2>Notes</h2><p></p><h2>Tasks</h2><ul data-type=\"taskList\"><li data-type=\"taskItem\" data-checked=\"false\"><label><input type=\"checkbox\"><span></span></label><div><p></p></div></li></ul><h2>Reflections</h2><p></p>",
        today
    );

    let id = Uuid::new_v4();
    let row = sqlx::query_as::<_, Node>(
        "INSERT INTO nodes (id, vault_id, title, content, content_type, word_count) VALUES ($1, $2, $3, $4, 'note', $5) RETURNING id, vault_id, title, content, content_type, file_path, metadata, word_count, created_at, updated_at",
    )
    .bind(id)
    .bind(vault_id)
    .bind(&today)
    .bind(&content)
    .bind(content.split_whitespace().count() as i32)
    .fetch_one(state.pg()?)
    .await
    .map_err(|e| format!("Failed to create daily note: {}", e))?;

    let _ = state
        .neo4j()?
        .run(
            neo4rs::query("CREATE (n:Node {pg_id: $pg_id, vault_id: $vault_id, title: $title, content_type: 'note'})")
                .param("pg_id", id.to_string())
                .param("vault_id", vault_id.to_string())
                .param("title", row.title.clone()),
        )
        .await;

    Ok(row)
}

#[tauri::command]
pub async fn list_templates(
    state: tauri::State<'_, AppState>,
    vault_id: Uuid,
) -> Result<Vec<NodeListItem>, String> {
    let nodes = sqlx::query_as::<_, NodeListItem>(
        "SELECT id, title, content_type, file_path, updated_at FROM nodes WHERE vault_id = $1 AND content_type = 'template' ORDER BY updated_at DESC",
    )
    .bind(vault_id)
    .fetch_all(state.pg()?)
    .await
    .map_err(|e| format!("Failed to list templates: {}", e))?;

    Ok(nodes)
}

#[tauri::command]
pub async fn create_node_from_template(
    state: tauri::State<'_, AppState>,
    template_id: Uuid,
    title: Option<String>,
) -> Result<Node, String> {
    let template = sqlx::query_as::<_, Node>(
        "SELECT id, vault_id, title, content, content_type, file_path, metadata, word_count, created_at, updated_at FROM nodes WHERE id = $1",
    )
    .bind(template_id)
    .fetch_one(state.pg()?)
    .await
    .map_err(|e| format!("Template not found: {}", e))?;

    let now = chrono::Utc::now();
    let new_title = title.unwrap_or_else(|| {
        template.title
            .replace("{{date}}", &now.format("%Y-%m-%d").to_string())
            .replace("{{time}}", &now.format("%H:%M").to_string())
    });

    let mut new_content = template.content
        .replace("{{date}}", &now.format("%Y-%m-%d").to_string())
        .replace("{{time}}", &now.format("%H:%M:%S").to_string())
        .replace("{{title}}", &new_title);

    if let Some(ref meta) = template.metadata {
        if let Some(tags) = meta.get("tags").and_then(|v| v.as_array()) {
            let tag_str: Vec<&str> = tags.iter().filter_map(|t| t.as_str()).collect();
            new_content = new_content.replace("{{tags}}", &tag_str.join(", "));
        }
    }

    let id = Uuid::new_v4();
    let row = sqlx::query_as::<_, Node>(
        "INSERT INTO nodes (id, vault_id, title, content, content_type, word_count) VALUES ($1, $2, $3, $4, 'note', $5) RETURNING id, vault_id, title, content, content_type, file_path, metadata, word_count, created_at, updated_at",
    )
    .bind(id)
    .bind(template.vault_id)
    .bind(&new_title)
    .bind(&new_content)
    .bind(new_content.split_whitespace().count() as i32)
    .fetch_one(state.pg()?)
    .await
    .map_err(|e| format!("Failed to create node from template: {}", e))?;

    let _ = state
        .neo4j()?
        .run(
            neo4rs::query("CREATE (n:Node {pg_id: $pg_id, vault_id: $vault_id, title: $title, content_type: 'note'})")
                .param("pg_id", id.to_string())
                .param("vault_id", template.vault_id.to_string())
                .param("title", row.title.clone()),
        )
        .await;

    Ok(row)
}
