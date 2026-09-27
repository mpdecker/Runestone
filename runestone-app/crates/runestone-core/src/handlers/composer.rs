use crate::context::BackendContext;
use crate::error::AppError;
use crate::handlers::{embeddings, node as node_handler, tag};
use crate::models::node::Node;
use crate::models::tag::AddTagsRequest;
use crate::repositories::node_repo;
use crate::services::graph_sync;
use crate::util::word_count;
use uuid::Uuid;

pub async fn merge_nodes(
    ctx: &BackendContext,
    source_id: Uuid,
    target_id: Uuid,
) -> Result<Node, String> {
    if source_id == target_id {
        // Merging a note into itself would append its own content and then delete it.
        return Err(
            AppError::Validation("Cannot merge a note into itself".to_string()).to_string(),
        );
    }

    let source = node_repo::get_by_id(&ctx.pg, source_id)
        .await
        .map_err(|e| e.to_string())?;

    let target = node_repo::get_by_id(&ctx.pg, target_id)
        .await
        .map_err(|e| e.to_string())?;

    if source.vault_id != target.vault_id {
        return Err(
            AppError::Validation("Cannot merge notes from different vaults".to_string())
                .to_string(),
        );
    }

    let merged_content = format!(
        "{}\n\n<hr>\n<h2>{}</h2>\n{}",
        target.content,
        crate::util::html_escape(&source.title),
        source.content
    );

    let mut tx = ctx
        .pg
        .begin()
        .await
        .map_err(|e| format!("Failed to start transaction: {}", e))?;

    // Keep the target's previous content recoverable.
    node_handler::insert_version(&mut tx, &target).await?;

    let updated = sqlx::query_as::<_, Node>(
        "UPDATE nodes SET content = $2, word_count = $3, updated_at = NOW() WHERE id = $1 RETURNING id, vault_id, title, content, content_type, file_path, metadata, word_count, created_at, updated_at",
    )
    .bind(target_id)
    .bind(&merged_content)
    .bind(word_count(&merged_content))
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| format!("Failed to merge: {}", e))?;

    // Links that pointed at the source now point at the target, and the source's own links
    // move over with its content (otherwise they vanish with the source's cascade delete).
    sqlx::query("UPDATE wiki_links SET resolved_node_id = $2 WHERE resolved_node_id = $1")
        .bind(source_id)
        .bind(target_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("Failed to re-point links: {}", e))?;
    sqlx::query(
        "INSERT INTO wiki_links (source_node_id, target_title, resolved_node_id, context) SELECT $2, target_title, resolved_node_id, context FROM wiki_links WHERE source_node_id = $1 ON CONFLICT (source_node_id, target_title) DO NOTHING",
    )
    .bind(source_id)
    .bind(target_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| format!("Failed to move links: {}", e))?;

    tx.commit()
        .await
        .map_err(|e| format!("Failed to commit merge: {}", e))?;

    graph_sync::update_node(&ctx.neo4j, target_id, &updated.title, &updated.content_type)
        .await
        .map_err(|e| format!("Neo4j update failed during merge: {}", e))?;

    graph_sync::move_links(&ctx.neo4j, source_id, target_id)
        .await
        .map_err(|e| format!("Neo4j link transfer failed during merge: {}", e))?;

    // Tags of the merged note are not lost either.
    let source_tags = tag::get_node_tags(ctx, source_id).await?.tags;
    if !source_tags.is_empty() {
        tag::add_tags_to_node(
            ctx,
            AddTagsRequest {
                node_id: target_id,
                tags: source_tags,
            },
        )
        .await?;
    }

    graph_sync::delete_node_before_pg(&ctx.neo4j, source_id)
        .await
        .map_err(|e| format!("Neo4j delete failed during merge: {}", e))?;

    node_repo::delete_by_id(&ctx.pg, source_id)
        .await
        .map_err(|e| e.to_string())?;

    embeddings::enqueue_embedding(ctx, target_id).await?;
    node_handler::sync_links_best_effort(ctx, &updated, false).await;

    Ok(updated)
}

/// Split editor HTML near its middle without cutting through a tag or a block element: the
/// split point is the end of the top-level block closest to the midpoint. Content with no
/// block structure is cut at the whitespace nearest the middle (always on a char boundary).
pub fn split_content(content: &str) -> (String, String) {
    static TAG: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let tag_re =
        TAG.get_or_init(|| regex::Regex::new(r"<(/?)([a-zA-Z][a-zA-Z0-9]*)[^>]*?(/?)>").unwrap());
    const VOID: [&str; 9] = [
        "br", "hr", "img", "input", "meta", "link", "col", "source", "wbr",
    ];

    let mid = content.len() / 2;
    let mut depth: i32 = 0;
    let mut best: Option<usize> = None;
    for cap in tag_re.captures_iter(content) {
        let m = cap.get(0).unwrap();
        let closing = !cap[1].is_empty();
        let self_closing = !cap[3].is_empty();
        let name = cap[2].to_ascii_lowercase();
        if VOID.contains(&name.as_str()) || self_closing {
            if depth == 0 {
                best = closer(best, m.end(), mid, content.len());
            }
            continue;
        }
        if closing {
            depth -= 1;
            if depth <= 0 {
                depth = 0;
                best = closer(best, m.end(), mid, content.len());
            }
        } else {
            depth += 1;
        }
    }
    if let Some(at) = best {
        return (content[..at].to_string(), content[at..].to_string());
    }

    // Fall back to whitespace near the middle, on a char boundary.
    let mut at = mid;
    while at > 0 && !content.is_char_boundary(at) {
        at -= 1;
    }
    if let Some(ws) = content[at..].find(char::is_whitespace) {
        at += ws;
    }
    (content[..at].to_string(), content[at..].to_string())
}

fn closer(best: Option<usize>, candidate: usize, mid: usize, len: usize) -> Option<usize> {
    if candidate == 0 || candidate >= len {
        return best;
    }
    match best {
        Some(b) if b.abs_diff(mid) <= candidate.abs_diff(mid) => Some(b),
        _ => Some(candidate),
    }
}

pub async fn split_node(
    ctx: &BackendContext,
    node_id: Uuid,
    new_title: String,
) -> Result<(Node, Node), String> {
    if new_title.trim().is_empty() {
        return Err(AppError::Validation("Title must not be empty".to_string()).to_string());
    }

    let mut tx = ctx
        .pg
        .begin()
        .await
        .map_err(|e| format!("Failed to start transaction: {}", e))?;

    let source = sqlx::query_as::<_, Node>(
        "SELECT id, vault_id, title, content, content_type, file_path, metadata, word_count, created_at, updated_at FROM nodes WHERE id = $1 FOR UPDATE",
    )
    .bind(node_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| format!("Failed to load node: {}", e))?
    .ok_or_else(|| AppError::NotFound(format!("Node {} not found", node_id)).to_string())?;

    let (first_content, second_content) = split_content(&source.content);
    node_handler::insert_version(&mut tx, &source).await?;

    let first = sqlx::query_as::<_, Node>(
        "UPDATE nodes SET content = $2, word_count = $3, updated_at = NOW() WHERE id = $1 RETURNING id, vault_id, title, content, content_type, file_path, metadata, word_count, created_at, updated_at",
    )
    .bind(node_id)
    .bind(&first_content)
    .bind(word_count(&first_content))
    .fetch_one(&mut *tx)
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
    .bind(word_count(&second_content))
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| format!("Failed to create split node: {}", e))?;

    tx.commit()
        .await
        .map_err(|e| format!("Failed to commit split: {}", e))?;

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

    node_handler::sync_links_best_effort(ctx, &first, false).await;
    node_handler::sync_links_best_effort(ctx, &second, true).await;
    embeddings::enqueue_embedding(ctx, node_id).await?;
    embeddings::enqueue_embedding(ctx, new_id).await?;

    Ok((first, second))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_content_cuts_between_top_level_blocks() {
        let html = "<p>first paragraph</p><p>second paragraph</p><p>third paragraph</p>";
        let (a, b) = split_content(html);
        assert_eq!(format!("{}{}", a, b), html);
        assert!(a.ends_with("</p>") && b.starts_with("<p>"), "{a} | {b}");
    }

    #[test]
    fn split_content_does_not_split_inside_nested_blocks() {
        let html =
            "<ul><li><p>one</p></li><li><p>two</p></li><li><p>three</p></li></ul><p>tail</p>";
        let (a, b) = split_content(html);
        assert_eq!(format!("{}{}", a, b), html);
        assert!(a == "<ul><li><p>one</p></li><li><p>two</p></li><li><p>three</p></li></ul>");
        assert_eq!(b, "<p>tail</p>");
    }

    #[test]
    fn split_content_handles_multibyte_text_without_panicking() {
        let text = "日本語のテキスト ".repeat(20);
        let (a, b) = split_content(&text);
        assert_eq!(format!("{}{}", a, b), text);
        assert!(!a.is_empty() && !b.is_empty());

        let html = "<p>日本語</p><p>テキスト</p><p>🙂🙂🙂</p>";
        let (a, b) = split_content(html);
        assert_eq!(format!("{}{}", a, b), html);
    }

    #[test]
    fn split_content_handles_tiny_or_empty_content() {
        assert_eq!(split_content(""), (String::new(), String::new()));
        let (a, b) = split_content("x");
        assert_eq!(format!("{}{}", a, b), "x");
    }
}
