use crate::error::AppResult;
use crate::models::graph::{GraphData, GraphEdge, GraphNode, GraphOptions};
use neo4rs::Graph;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

const DEFAULT_GRAPH_NODE_LIMIT: usize = 500;

pub async fn fetch_graph_data(
    graph: &Arc<Graph>,
    vault_id: Uuid,
    options: Option<GraphOptions>,
) -> AppResult<GraphData> {
    let vault_id_str = vault_id.to_string();
    let tag_filter = options.as_ref().and_then(|o| o.tag.clone());

    let mut node_map: HashMap<String, GraphNode> = HashMap::new();
    let mut edges: Vec<GraphEdge> = Vec::new();

    if let Some(ref tag) = tag_filter {
        let node_query =
            "MATCH (n:Node {vault_id: $vault_id})-[:HAS_TAG]->(t:Tag {name: $tag}) RETURN n.pg_id, n.title, n.content_type";
        let mut node_stream = graph
            .execute(
                neo4rs::query(node_query)
                    .param("vault_id", vault_id_str.clone())
                    .param("tag", tag.as_str()),
            )
            .await?;
        while let Ok(Some(row)) = node_stream.next().await {
            let pg_id: String = row.get("n.pg_id")?;
            let title: String = row.get("n.title")?;
            let content_type: String = row.get("n.content_type")?;
            node_map.insert(
                pg_id.clone(),
                GraphNode {
                    id: pg_id,
                    title,
                    content_type,
                },
            );
        }
        let edge_query = "MATCH (n:Node {vault_id: $vault_id})-[:HAS_TAG]->(t:Tag {name: $tag}) MATCH (n)-[r]->(m:Node)-[:HAS_TAG]->(t) RETURN n.pg_id, type(r), m.pg_id";
        let mut edge_stream = graph
            .execute(
                neo4rs::query(edge_query)
                    .param("vault_id", vault_id_str)
                    .param("tag", tag.as_str()),
            )
            .await?;
        while let Ok(Some(row)) = edge_stream.next().await {
            edges.push(GraphEdge {
                source: row.get("n.pg_id")?,
                target: row.get("m.pg_id")?,
                label: row.get("type(r)")?,
            });
        }
    } else {
        let node_query = "MATCH (n:Node {vault_id: $vault_id}) RETURN n.pg_id, n.title, n.content_type";
        let mut node_stream = graph
            .execute(neo4rs::query(node_query).param("vault_id", vault_id_str.clone()))
            .await?;
        while let Ok(Some(row)) = node_stream.next().await {
            let pg_id: String = row.get("n.pg_id")?;
            let title: String = row.get("n.title")?;
            let content_type: String = row.get("n.content_type")?;
            node_map.insert(
                pg_id.clone(),
                GraphNode {
                    id: pg_id,
                    title,
                    content_type,
                },
            );
        }
        let edge_query = "MATCH (n:Node {vault_id: $vault_id})-[r]->(m:Node {vault_id: $vault_id}) RETURN n.pg_id, type(r), m.pg_id";
        let mut edge_stream = graph
            .execute(neo4rs::query(edge_query).param("vault_id", vault_id_str))
            .await?;
        while let Ok(Some(row)) = edge_stream.next().await {
            edges.push(GraphEdge {
                source: row.get("n.pg_id")?,
                target: row.get("m.pg_id")?,
                label: row.get("type(r)")?,
            });
        }
    }

    let mut nodes: Vec<GraphNode> = node_map.into_values().collect();
    if nodes.len() > DEFAULT_GRAPH_NODE_LIMIT {
        nodes.sort_by(|a, b| a.title.cmp(&b.title));
        nodes.truncate(DEFAULT_GRAPH_NODE_LIMIT);
        let kept: std::collections::HashSet<String> =
            nodes.iter().map(|n| n.id.clone()).collect();
        edges.retain(|e| kept.contains(&e.source) && kept.contains(&e.target));
    }

    Ok(GraphData { nodes, edges })
}
