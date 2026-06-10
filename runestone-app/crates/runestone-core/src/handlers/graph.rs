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

    let re = regex::Regex::new(r"\[\[([^\]]+)\]\]").unwrap();
    let mut created_links: Vec<WikiLinkRow> = Vec::new();

    for cap in re.captures_iter(&node.content) {
        let raw_target = cap[1].to_string();
        let (target_title, block_ref) = if let Some(pos) = raw_target.find('#') {
            let (title_part, ref_part) = raw_target.split_at(pos);
            (title_part.to_string(), Some(ref_part.to_string()))
        } else if let Some(pos) = raw_target.find('^') {
            let (title_part, ref_part) = raw_target.split_at(pos);
            (title_part.to_string(), Some(ref_part.to_string()))
        } else {
            (raw_target.clone(), None)
        };

        let existing = sqlx::query_as::<_, WikiLinkRow>(
            "SELECT id, source_node_id, target_title, resolved_node_id, context, created_at FROM wiki_links WHERE source_node_id = $1 AND target_title = $2",
        )
        .bind(node_id)
        .bind(&target_title)
        .fetch_optional(&ctx.pg)
        .await
        .map_err(|e| format!("Query error: {}", e))?;

        if existing.is_some() {
            continue;
        }

        let resolved = sqlx::query_as::<_, NodeIdRow>(
            "SELECT id FROM nodes WHERE vault_id = $1 AND title = $2 LIMIT 1",
        )
        .bind(node.vault_id)
        .bind(&target_title)
        .fetch_optional(&ctx.pg)
        .await
        .map_err(|e| format!("Query error: {}", e))?;

        let id = Uuid::new_v4();
        let resolved_id = resolved.map(|r| r.id);

        let link = sqlx::query_as::<_, WikiLinkRow>(
            "INSERT INTO wiki_links (id, source_node_id, target_title, resolved_node_id, context) VALUES ($1, $2, $3, $4, $5) RETURNING id, source_node_id, target_title, resolved_node_id, context, created_at",
        )
        .bind(id)
        .bind(node_id)
        .bind(&target_title)
        .bind(resolved_id)
        .bind(&block_ref)
        .fetch_one(&ctx.pg)
        .await
        .map_err(|e| format!("Failed to insert wiki link: {}", e))?;

        if let Some(rid) = resolved_id {
            graph_sync::create_wiki_link(&ctx.neo4j, node_id, rid)
                .await
                .map_err(|e| format!("Neo4j wiki link failed: {}", e))?;
        }

        created_links.push(link);
    }

    Ok(created_links)
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
            let ctx_snippet = s.content.chars().take(100).collect::<String>();
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
            neo4rs::query("MATCH (n:Node)-[:LINKS_TO]->(target:Node {pg_id: $pg_id}) RETURN n.pg_id")
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
                    let ctx_snippet = s.content.chars().take(100).collect::<String>();
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
                let ctx_snippet = t.content.chars().take(100).collect::<String>();
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
                    let ctx_snippet = t.content.chars().take(100).collect::<String>();
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
