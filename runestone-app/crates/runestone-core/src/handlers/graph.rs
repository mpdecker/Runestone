use crate::context::BackendContext;
use crate::models::graph::{Backlink, GraphData, GraphEdge, GraphNode, GraphOptions, WikiLinkRow};
use crate::models::node::{Node, NodeIdRow};
use crate::repositories::graph_repo;
use crate::services::graph_sync;
use uuid::Uuid;

pub async fn get_graph_data(
    ctx: &BackendContext,
    vault_id: Uuid,
    options: Option<GraphOptions>,
) -> Result<GraphData, String> {
    graph_repo::fetch_graph_data(&ctx.neo4j, vault_id, options)
        .await
        .map_err(|e| e.to_string())
}

pub async fn get_local_graph(
    ctx: &BackendContext,
    node_id: Uuid,
    depth: Option<u32>,
) -> Result<GraphData, String> {
    let pg_id = node_id.to_string();
    let hop_depth = depth.unwrap_or(1).min(5);

    let mut node_map: std::collections::HashMap<String, GraphNode> =
        std::collections::HashMap::new();
    let mut edges: Vec<GraphEdge> = Vec::new();

    let node_query = format!(
        "MATCH (center:Node {{pg_id: $pg_id}})-[r*1..{}]-(neighbor:Node) RETURN center.pg_id, center.title, center.content_type, neighbor.pg_id, neighbor.title, neighbor.content_type",
        hop_depth
    );
    let mut stream = ctx
        .neo4j
        .execute(neo4rs::query(&node_query).param("pg_id", pg_id.clone()))
        .await
        .map_err(|e| format!("Neo4j local graph query failed: {}", e))?;

    while let Ok(Some(row)) = stream.next().await {
        let c_pg_id: String = row
            .get("center.pg_id")
            .map_err(|e: neo4rs::DeError| format!("center.pg_id: {}", e))?;
        let c_title: String = row
            .get("center.title")
            .map_err(|e: neo4rs::DeError| format!("center.title: {}", e))?;
        let c_type: String = row
            .get("center.content_type")
            .map_err(|e: neo4rs::DeError| format!("center.content_type: {}", e))?;
        node_map.insert(
            c_pg_id.clone(),
            GraphNode {
                id: c_pg_id.clone(),
                title: c_title,
                content_type: c_type,
            },
        );

        let n_pg_id: String = row
            .get("neighbor.pg_id")
            .map_err(|e: neo4rs::DeError| format!("neighbor.pg_id: {}", e))?;
        let n_title: String = row
            .get("neighbor.title")
            .map_err(|e: neo4rs::DeError| format!("neighbor.title: {}", e))?;
        let n_type: String = row
            .get("neighbor.content_type")
            .map_err(|e: neo4rs::DeError| format!("neighbor.content_type: {}", e))?;
        node_map.insert(
            n_pg_id.clone(),
            GraphNode {
                id: n_pg_id.clone(),
                title: n_title,
                content_type: n_type,
            },
        );
    }

    let edge_query = format!(
        "MATCH (center:Node {{pg_id: $pg_id}})-[r*1..{}]-(neighbor:Node) WITH center, neighbor MATCH (center)-[direct_r]->(neighbor) WHERE NOT (neighbor)-[:LINKS_TO]->(center) RETURN center.pg_id, type(direct_r), neighbor.pg_id UNION MATCH (center:Node {{pg_id: $pg_id}})-[r*1..{}]-(neighbor:Node) WITH center, neighbor MATCH (neighbor)-[direct_r]->(center) WHERE NOT (center)-[:LINKS_TO]->(neighbor) RETURN neighbor.pg_id, type(direct_r), center.pg_id",
        hop_depth, hop_depth
    );
    let mut edge_stream = ctx
        .neo4j
        .execute(neo4rs::query(&edge_query).param("pg_id", pg_id))
        .await
        .map_err(|e| format!("Neo4j local graph edge query failed: {}", e))?;

    while let Ok(Some(row)) = edge_stream.next().await {
        let source: String = row
            .get("center.pg_id")
            .map_err(|e: neo4rs::DeError| format!("center.pg_id: {}", e))?;
        let target: String = row
            .get("neighbor.pg_id")
            .map_err(|e: neo4rs::DeError| format!("neighbor.pg_id: {}", e))?;
        let rel_type: String = row
            .get("type(direct_r)")
            .map_err(|e: neo4rs::DeError| format!("type(direct_r): {}", e))?;
        edges.push(GraphEdge {
            source,
            target,
            label: rel_type,
        });
    }

    Ok(GraphData {
        nodes: node_map.into_values().collect(),
        edges,
    })
}

/// One `[[...]]` occurrence, with the titles it may refer to in resolution order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedLink {
    /// `(title, context)` candidates. The first is the whole link text (minus any `|alias`) so
    /// notes whose titles contain `#` or `^` still resolve; the last is the text before the
    /// first `#`/`^` with the heading/block reference as context.
    pub candidates: Vec<(String, Option<String>)>,
}

/// Extract `[[wiki links]]` from note content. Content is editor HTML, so link text arrives
/// HTML-escaped (`[[A &amp; B]]`); decode before matching so it can resolve to `A & B`.
pub fn extract_wiki_links(content: &str) -> Vec<ParsedLink> {
    static RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let re = RE.get_or_init(|| regex::Regex::new(r"\[\[([^\]]+)\]\]").unwrap());

    let mut links = Vec::new();
    for cap in re.captures_iter(content) {
        let decoded = crate::util::decode_html_entities(&cap[1]);
        // `[[Title|alias]]`: the alias is display text only.
        let target = decoded.split('|').next().unwrap_or("").trim().to_string();
        if target.is_empty() {
            continue;
        }
        let mut candidates = vec![(target.clone(), None)];
        if let Some(pos) = target.find(['#', '^']) {
            let (title_part, ref_part) = target.split_at(pos);
            let title_part = title_part.trim();
            if !title_part.is_empty() {
                candidates.push((title_part.to_string(), Some(ref_part.to_string())));
            }
        }
        links.push(ParsedLink { candidates });
    }
    links
}

async fn resolve_title(
    pool: &sqlx::PgPool,
    vault_id: Uuid,
    title: &str,
) -> Result<Option<Uuid>, String> {
    let row = sqlx::query_as::<_, NodeIdRow>(
        "SELECT id FROM nodes WHERE vault_id = $1 AND (title = $2 OR lower(title) = lower($2) OR metadata->'aliases' ? $2) ORDER BY (title = $2) DESC, created_at LIMIT 1",
    )
    .bind(vault_id)
    .bind(title)
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("Query error: {}", e))?;
    Ok(row.map(|r| r.id))
}

/// Re-derive a note's wiki links from its content so PostgreSQL (`wiki_links`) and Neo4j
/// (`LINKS_TO`) both match what the note says *now*: new links are added, links that were
/// deleted from the text are removed, and links whose target did not exist yet are resolved.
/// Returns the rows that were inserted or changed.
pub async fn parse_wiki_links(
    ctx: &BackendContext,
    node_id: Uuid,
) -> Result<Vec<WikiLinkRow>, String> {
    let node = sqlx::query_as::<_, Node>(
        "SELECT id, vault_id, title, content, content_type, file_path, metadata, word_count, created_at, updated_at FROM nodes WHERE id = $1",
    )
    .bind(node_id)
    .fetch_one(&ctx.pg)
    .await
    .map_err(|e| format!("Node not found: {}", e))?;

    // Desired state: (stored title, context, resolved target), first occurrence wins.
    let mut desired: Vec<(String, Option<String>, Option<Uuid>)> = Vec::new();
    for link in extract_wiki_links(&node.content) {
        let mut chosen = None;
        for (title, context) in &link.candidates {
            if let Some(id) = resolve_title(&ctx.pg, node.vault_id, title).await? {
                chosen = Some((title.clone(), context.clone(), Some(id)));
                break;
            }
        }
        let (title, context, resolved) = chosen.unwrap_or_else(|| {
            let (t, c) = link.candidates.last().cloned().unwrap();
            (t, c, None)
        });
        if !desired.iter().any(|(t, _, _)| *t == title) {
            desired.push((title, context, resolved));
        }
    }

    let existing = sqlx::query_as::<_, WikiLinkRow>(
        "SELECT id, source_node_id, target_title, resolved_node_id, context, created_at FROM wiki_links WHERE source_node_id = $1",
    )
    .bind(node_id)
    .fetch_all(&ctx.pg)
    .await
    .map_err(|e| format!("Query error: {}", e))?;

    let mut changed: Vec<WikiLinkRow> = Vec::new();
    let mut edges_to_keep: Vec<Uuid> = Vec::new();

    for (title, context, resolved) in &desired {
        match existing.iter().find(|r| r.target_title == *title) {
            Some(row) => {
                // Keep an existing resolution when the text no longer resolves (e.g. the target
                // note was renamed); otherwise follow the current resolution.
                let target = resolved.or(row.resolved_node_id);
                if let Some(t) = target {
                    edges_to_keep.push(t);
                }
                if target != row.resolved_node_id {
                    let updated = sqlx::query_as::<_, WikiLinkRow>(
                        "UPDATE wiki_links SET resolved_node_id = $2, context = $3 WHERE id = $1 RETURNING id, source_node_id, target_title, resolved_node_id, context, created_at",
                    )
                    .bind(row.id)
                    .bind(target)
                    .bind(context)
                    .fetch_one(&ctx.pg)
                    .await
                    .map_err(|e| format!("Failed to update wiki link: {}", e))?;
                    changed.push(updated);
                }
            }
            None => {
                let link = sqlx::query_as::<_, WikiLinkRow>(
                    "INSERT INTO wiki_links (id, source_node_id, target_title, resolved_node_id, context) VALUES ($1, $2, $3, $4, $5) ON CONFLICT (source_node_id, target_title) DO NOTHING RETURNING id, source_node_id, target_title, resolved_node_id, context, created_at",
                )
                .bind(Uuid::new_v4())
                .bind(node_id)
                .bind(title)
                .bind(resolved)
                .bind(context)
                .fetch_optional(&ctx.pg)
                .await
                .map_err(|e| format!("Failed to insert wiki link: {}", e))?;
                if let Some(link) = link {
                    if let Some(t) = resolved {
                        edges_to_keep.push(*t);
                    }
                    changed.push(link);
                }
            }
        }
    }

    // Links that are no longer in the text.
    for row in existing
        .iter()
        .filter(|r| !desired.iter().any(|(t, _, _)| *t == r.target_title))
    {
        sqlx::query("DELETE FROM wiki_links WHERE id = $1")
            .bind(row.id)
            .execute(&ctx.pg)
            .await
            .map_err(|e| format!("Failed to remove stale wiki link: {}", e))?;
    }

    // Neo4j: drop edges that no remaining link justifies, (re)create the ones that are.
    let stale_targets: Vec<Uuid> = existing
        .iter()
        .filter_map(|r| r.resolved_node_id)
        .filter(|t| !edges_to_keep.contains(t))
        .collect();
    for target in stale_targets {
        graph_sync::delete_wiki_link(&ctx.neo4j, node_id, target)
            .await
            .map_err(|e| format!("Neo4j wiki link removal failed: {}", e))?;
    }
    for target in edges_to_keep {
        graph_sync::create_wiki_link(&ctx.neo4j, node_id, target)
            .await
            .map_err(|e| format!("Neo4j wiki link failed: {}", e))?;
    }

    Ok(changed)
}

/// After a note is created or renamed, links elsewhere in the vault that were waiting for that
/// title ("[[Later]]" written before "Later" existed) now resolve to it.
pub async fn resolve_pending_links(
    ctx: &BackendContext,
    node_id: Uuid,
    vault_id: Uuid,
    title: &str,
) -> Result<(), String> {
    if title.trim().is_empty() {
        return Ok(());
    }
    let sources = sqlx::query_as::<_, (Uuid,)>(
        "UPDATE wiki_links wl SET resolved_node_id = $1 FROM nodes src WHERE wl.source_node_id = src.id AND src.vault_id = $2 AND wl.resolved_node_id IS NULL AND lower(wl.target_title) = lower($3) RETURNING wl.source_node_id",
    )
    .bind(node_id)
    .bind(vault_id)
    .bind(title)
    .fetch_all(&ctx.pg)
    .await
    .map_err(|e| format!("Failed to resolve pending links: {}", e))?;

    for (source_id,) in sources {
        graph_sync::create_wiki_link(&ctx.neo4j, source_id, node_id)
            .await
            .map_err(|e| format!("Neo4j wiki link failed: {}", e))?;
    }
    Ok(())
}

pub async fn get_backlinks(ctx: &BackendContext, node_id: Uuid) -> Result<Vec<Backlink>, String> {
    let pg_id = node_id.to_string();
    let mut backlinks = Vec::new();

    let pg_links = sqlx::query_as::<_, WikiLinkRow>(
        "SELECT id, source_node_id, target_title, resolved_node_id, context, created_at FROM wiki_links WHERE resolved_node_id = $1",
    )
    .bind(node_id)
    .fetch_all(&ctx.pg)
    .await
    .map_err(|e| format!("Query error: {}", e))?;

    for link in pg_links {
        let source = sqlx::query_as::<_, Node>(
            "SELECT id, vault_id, title, content, content_type, file_path, metadata, word_count, created_at, updated_at FROM nodes WHERE id = $1",
        )
        .bind(link.source_node_id)
        .fetch_optional(&ctx.pg)
        .await
        .map_err(|e| format!("Query error: {}", e))?;

        if let Some(s) = source {
            let ctx_snippet =
                crate::util::truncate_chars(&crate::util::plain_text(&s.content), 100).to_string();
            backlinks.push(Backlink {
                node_id: s.id,
                title: s.title,
                content_type: s.content_type,
                context: Some(ctx_snippet),
            });
        }
    }

    let mut stream = ctx
        .neo4j
        .execute(
            neo4rs::query(
                "MATCH (n:Node)-[:LINKS_TO]->(target:Node {pg_id: $pg_id}) RETURN n.pg_id",
            )
            .param("pg_id", pg_id),
        )
        .await
        .map_err(|e| format!("Neo4j query failed: {}", e))?;

    while let Ok(Some(row)) = stream.next().await {
        let linked_pg_id: String = row.get("n.pg_id").unwrap_or_default();
        if let Ok(uid) = Uuid::parse_str(&linked_pg_id) {
            if !backlinks.iter().any(|b| b.node_id == uid) {
                let source = sqlx::query_as::<_, Node>(
                    "SELECT id, vault_id, title, content, content_type, file_path, metadata, word_count, created_at, updated_at FROM nodes WHERE id = $1",
                )
                .bind(uid)
                .fetch_optional(&ctx.pg)
                .await
                .map_err(|e| format!("Query error: {}", e))?;

                if let Some(s) = source {
                    let ctx_snippet =
                        crate::util::truncate_chars(&crate::util::plain_text(&s.content), 100)
                            .to_string();
                    backlinks.push(Backlink {
                        node_id: s.id,
                        title: s.title,
                        content_type: s.content_type,
                        context: Some(ctx_snippet),
                    });
                }
            }
        }
    }

    Ok(backlinks)
}

pub async fn get_outgoing_links(
    ctx: &BackendContext,
    node_id: Uuid,
) -> Result<Vec<Backlink>, String> {
    let mut outgoing = Vec::new();

    let pg_links = sqlx::query_as::<_, WikiLinkRow>(
        "SELECT id, source_node_id, target_title, resolved_node_id, context, created_at FROM wiki_links WHERE source_node_id = $1",
    )
    .bind(node_id)
    .fetch_all(&ctx.pg)
    .await
    .map_err(|e| format!("Query error: {}", e))?;

    for link in pg_links {
        if let Some(resolved_id) = link.resolved_node_id {
            let target = sqlx::query_as::<_, Node>(
                "SELECT id, vault_id, title, content, content_type, file_path, metadata, word_count, created_at, updated_at FROM nodes WHERE id = $1",
            )
            .bind(resolved_id)
            .fetch_optional(&ctx.pg)
            .await
            .map_err(|e| format!("Query error: {}", e))?;

            if let Some(t) = target {
                let ctx_snippet =
                    crate::util::truncate_chars(&crate::util::plain_text(&t.content), 100)
                        .to_string();
                outgoing.push(Backlink {
                    node_id: t.id,
                    title: t.title,
                    content_type: t.content_type,
                    context: Some(ctx_snippet),
                });
            }
        } else {
            outgoing.push(Backlink {
                node_id: Uuid::nil(),
                title: link.target_title.clone(),
                content_type: "unresolved".to_string(),
                context: None,
            });
        }
    }

    let mut stream = ctx
        .neo4j
        .execute(
            neo4rs::query(
                "MATCH (n:Node {pg_id: $pg_id})-[r:LINKS_TO]->(target:Node) RETURN target.pg_id",
            )
            .param("pg_id", node_id.to_string()),
        )
        .await
        .map_err(|e| format!("Neo4j query failed: {}", e))?;

    while let Ok(Some(row)) = stream.next().await {
        let linked_pg_id: String = row.get("target.pg_id").unwrap_or_default();
        if let Ok(uid) = Uuid::parse_str(&linked_pg_id) {
            if !outgoing.iter().any(|b| b.node_id == uid) {
                let target = sqlx::query_as::<_, Node>(
                    "SELECT id, vault_id, title, content, content_type, file_path, metadata, word_count, created_at, updated_at FROM nodes WHERE id = $1",
                )
                .bind(uid)
                .fetch_optional(&ctx.pg)
                .await
                .map_err(|e| format!("Query error: {}", e))?;

                if let Some(t) = target {
                    let ctx_snippet =
                        crate::util::truncate_chars(&crate::util::plain_text(&t.content), 100)
                            .to_string();
                    outgoing.push(Backlink {
                        node_id: t.id,
                        title: t.title,
                        content_type: t.content_type,
                        context: Some(ctx_snippet),
                    });
                }
            }
        }
    }

    Ok(outgoing)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn titles(link: &ParsedLink) -> Vec<&str> {
        link.candidates.iter().map(|(t, _)| t.as_str()).collect()
    }

    #[test]
    fn extracts_plain_and_html_wrapped_links() {
        let links = extract_wiki_links(
            r#"<p>see <span data-type="wiki-link" data-title="Beta">[[Beta]]</span> and [[Gamma]]</p>"#,
        );
        assert_eq!(links.len(), 2);
        assert_eq!(titles(&links[0]), vec!["Beta"]);
        assert_eq!(titles(&links[1]), vec!["Gamma"]);
    }

    #[test]
    fn decodes_html_entities_in_link_text() {
        let links = extract_wiki_links("<p>[[A &amp; B]] [[Tom &quot;T&quot; &lt;x&gt;]]</p>");
        assert_eq!(titles(&links[0]), vec!["A & B"]);
        assert_eq!(titles(&links[1]), vec!["Tom \"T\" <x>"]);
    }

    #[test]
    fn strips_alias_and_splits_heading_or_block_refs() {
        let links =
            extract_wiki_links("[[Beta|shown text]] [[Beta#Heading]] [[Beta^blk]] [[C# notes]]");
        assert_eq!(links[0].candidates, vec![("Beta".to_string(), None)]);
        assert_eq!(
            links[1].candidates,
            vec![
                ("Beta#Heading".to_string(), None),
                ("Beta".to_string(), Some("#Heading".to_string()))
            ]
        );
        assert_eq!(
            links[2].candidates.last().unwrap().1.as_deref(),
            Some("^blk")
        );
        // a title that itself contains '#': the whole text is tried first
        assert_eq!(titles(&links[3]), vec!["C# notes", "C"]);
    }

    #[test]
    fn ignores_empty_links() {
        assert!(extract_wiki_links("[[|alias]] [[ ]]").is_empty());
    }
}
