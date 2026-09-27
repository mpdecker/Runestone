use crate::context::BackendContext;
use crate::error::{AppError, AppResult};
use crate::models::graph::{CypherResultRow, GraphQueryRequest, GraphQueryResponse};

const MAX_CYPHER_ROWS: usize = 200;
const MAX_RESULT_CHARS: usize = 4000;

/// Clauses/keywords that write, or that reach outside the graph. Matched as whole words on the
/// query with string literals and comments removed, so `WHERE n.title = 'Set theory'`,
/// `created_at` or `Dataset` are not mistaken for `SET`/`CREATE`.
const DESTRUCTIVE_KEYWORDS: &[&str] = &[
    "DELETE", "DETACH", "REMOVE", "SET", "CREATE", "MERGE", "DROP", "CALL", "FOREACH", "LOAD",
    "PERIODIC", "COMMIT", "APOC", "DBMS",
];

pub async fn run_cypher(ctx: &BackendContext, cypher: String) -> AppResult<Vec<CypherResultRow>> {
    let sanitized = sanitize_cypher(&cypher)?;

    log::debug!("Executing Cypher query: {}", sanitized);
    let query = neo4rs::query(&sanitized);
    let mut stream = ctx
        .neo4j
        .execute(query)
        .await
        .map_err(|e| AppError::Neo4j(format!("Cypher execution failed: {}", e)))?;

    let mut rows: Vec<CypherResultRow> = Vec::new();
    loop {
        // Surface stream errors (e.g. a rejected write) instead of returning a silent "no rows".
        let row = match stream.next().await {
            Ok(Some(row)) => row,
            Ok(None) => break,
            Err(e) => return Err(AppError::Neo4j(format!("Cypher execution failed: {}", e))),
        };
        if rows.len() >= MAX_CYPHER_ROWS {
            break;
        }
        let mut pairs: Vec<(String, String)> = Vec::new();
        if let Ok(map) = row.to::<serde_json::Map<String, serde_json::Value>>() {
            for (key, val) in map {
                let text = match val {
                    serde_json::Value::Null => continue,
                    serde_json::Value::String(s) => s,
                    other => other.to_string(),
                };
                if !text.is_empty() {
                    pairs.push((key, text));
                }
            }
        }
        if pairs.is_empty() {
            pairs.push(("result".to_string(), "1 match".to_string()));
        }
        rows.push(CypherResultRow { values: pairs });
    }

    Ok(rows)
}

/// Returns `(executable, inspectable)`: the query without comments, and the same text with
/// string literals blanked out (used only for keyword checks).
fn strip_comments_and_strings(cypher: &str) -> (String, String) {
    let chars: Vec<char> = cypher.chars().collect();
    let mut exec = String::with_capacity(cypher.len());
    let mut check = String::with_capacity(cypher.len());
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        let next = chars.get(i + 1).copied();
        if c == '/' && next == Some('/') {
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
            exec.push('\n');
            check.push('\n');
        } else if c == '/' && next == Some('*') {
            i += 2;
            while i + 1 < chars.len() && !(chars[i] == '*' && chars[i + 1] == '/') {
                i += 1;
            }
            i += 2;
            exec.push(' ');
            check.push(' ');
        } else if c == '\'' || c == '"' {
            let quote = c;
            exec.push(c);
            check.push(' ');
            i += 1;
            while i < chars.len() {
                let d = chars[i];
                exec.push(d);
                if d == '\\' && i + 1 < chars.len() {
                    exec.push(chars[i + 1]);
                    i += 2;
                    continue;
                }
                i += 1;
                if d == quote {
                    break;
                }
            }
            check.push(' ');
        } else {
            exec.push(c);
            check.push(c);
            i += 1;
        }
    }
    (exec, check)
}

fn sanitize_cypher(cypher: &str) -> AppResult<String> {
    let (exec, check) = strip_comments_and_strings(cypher);
    let upper = check.to_uppercase();

    for word in upper.split(|c: char| !(c.is_alphanumeric() || c == '_')) {
        if DESTRUCTIVE_KEYWORDS.contains(&word) {
            return Err(AppError::Validation(format!(
                "Cypher query contains disallowed keyword: '{}'. Only read-only queries are permitted.",
                word
            )));
        }
    }

    let trimmed = exec.trim().trim_end_matches(';').trim();
    let has_limit = upper
        .split(|c: char| !(c.is_alphanumeric() || c == '_'))
        .any(|w| w == "LIMIT");
    Ok(if has_limit {
        trimmed.to_string()
    } else {
        format!("{} LIMIT {}", trimmed, MAX_CYPHER_ROWS)
    })
}

pub async fn graph_query(
    ctx: &BackendContext,
    request: GraphQueryRequest,
) -> AppResult<GraphQueryResponse> {
    let schema = r#"Node labels: Node (properties: pg_id, title, content_type).
Edge types: LINKS_TO, HAS_TAG, REFERENCES, CONTAINS, EXTRACTED_FROM, RELATES_TO.
pg_id is a UUID matching the PostgreSQL nodes.id column.
content_type values: note, concept, entity, document."#;

    let vault_nodes_hint = format!(
        "All nodes for this vault have pg_ids that match nodes with vault_id '{}' in PostgreSQL.",
        request.vault_id
    );

    let cypher_prompt = format!(
        "You are a Neo4j Cypher translator. Given the following graph schema and a user's question, output ONLY a valid Cypher query (no markdown, no explanation, no backticks) that answers the question. Use only read-only clauses (MATCH, RETURN, WHERE, ORDER BY, SKIP, LIMIT).\n\nSchema:\n{}\n\nAdditional context:\n{}\n\nUser's question: {}\n\nCypher query:",
        schema, vault_nodes_hint, request.question
    );

    let cypher = crate::handlers::ai::simple_chat(&cypher_prompt, &ctx.llm_config)
        .await
        .map_err(|e| AppError::Other(format!("LLM translation failed: {}", e)))?;

    let cleaned_cypher = clean_cypher(&cypher);
    log::info!("Generated Cypher from NL query: {}", cleaned_cypher);

    let results = run_cypher(ctx, cleaned_cypher.clone()).await?;
    log::info!("Cypher query completed: {} rows", results.len());

    let results_text = if results.is_empty() {
        "The query returned no results.".to_string()
    } else {
        let rows_text: Vec<String> = results
            .iter()
            .map(|r| {
                r.values
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v))
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .collect();
        let joined = rows_text.join("\n");
        if joined.len() > MAX_RESULT_CHARS {
            format!(
                "{}...\n(truncated, {} total rows)",
                crate::util::truncate_bytes(&joined, MAX_RESULT_CHARS),
                results.len()
            )
        } else {
            joined
        }
    };

    let answer_prompt = format!(
        "Given these Cypher query results from a knowledge graph:\n{}\n\nAnswer the user's original question concisely: {}\n\nAnswer:",
        results_text, request.question,
    );

    let answer = crate::handlers::ai::simple_chat(&answer_prompt, &ctx.llm_config)
        .await
        .map_err(|e| AppError::Other(format!("LLM summarization failed: {}", e)))?;

    Ok(GraphQueryResponse {
        answer,
        cypher: cleaned_cypher,
        results,
    })
}

fn clean_cypher(raw: &str) -> String {
    let trimmed = raw.trim();
    let stripped = trimmed
        .trim_start_matches("```cypher")
        .trim_start_matches("```cypher\n")
        .trim_start_matches("```")
        .trim_start_matches("cypher")
        .trim_end_matches("```")
        .trim();
    if stripped.is_empty() {
        raw.trim().to_string()
    } else {
        stripped.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_allows_read_only() {
        assert!(sanitize_cypher("MATCH (n) RETURN n").is_ok());
        assert!(sanitize_cypher("MATCH (n:Node {pg_id: 'x'})-[r]-(m) RETURN n, m").is_ok());
    }

    #[test]
    fn sanitize_blocks_delete() {
        assert!(sanitize_cypher("MATCH (n) DELETE n").is_err());
        assert!(sanitize_cypher("MATCH (n) DETACH DELETE n").is_err());
    }

    #[test]
    fn sanitize_blocks_destructive_write() {
        assert!(sanitize_cypher("CREATE (n:Node {pg_id: 'x'})").is_err());
        assert!(sanitize_cypher("DROP CONSTRAINT ON (n:Node)").is_err());
        assert!(sanitize_cypher("MATCH (n) REMOVE n.title").is_err());
        assert!(sanitize_cypher("MATCH (n) SET n.title = 'x'").is_err());
    }

    #[test]
    fn sanitize_adds_limit_when_missing() {
        let result = sanitize_cypher("MATCH (n) RETURN n").unwrap();
        assert!(result.contains("LIMIT"));
    }

    #[test]
    fn sanitize_does_not_flag_words_that_merely_contain_keywords() {
        assert!(
            sanitize_cypher("MATCH (n:Node) WHERE n.title CONTAINS 'Dataset' RETURN n.title")
                .is_ok()
        );
        assert!(
            sanitize_cypher("MATCH (n:Node) WHERE n.title = 'Set theory' RETURN n.pg_id").is_ok()
        );
        assert!(
            sanitize_cypher("MATCH (n:Node) WHERE n.created_at IS NULL RETURN n.pg_id").is_ok()
        );
        assert!(sanitize_cypher("MATCH (n:Node) RETURN n.title ORDER BY n.title SKIP 1").is_ok());
        assert!(sanitize_cypher("MATCH (n:Node) RETURN n.pg_id AS reset_value").is_ok());
    }

    #[test]
    fn sanitize_blocks_write_and_procedure_keywords_in_any_case_or_spacing() {
        assert!(sanitize_cypher("match (n) detach   delete n").is_err());
        assert!(sanitize_cypher(
            "MATCH (n)
SET n.x = 1"
        )
        .is_err());
        assert!(sanitize_cypher("MATCH (n) FOREACH (x IN [1] | CREATE (:A))").is_err());
        assert!(sanitize_cypher("CALL db.labels()").is_err());
        assert!(sanitize_cypher("LOAD CSV FROM 'file:///x' AS r RETURN r").is_err());
        assert!(sanitize_cypher("RETURN apoc.cypher.runFirstColumnSingle('x', {})").is_err());
        // a keyword hidden after a comment marker is still checked, a commented-out one is not
        assert!(sanitize_cypher("MATCH (n) RETURN n // DELETE").is_ok());
        assert!(sanitize_cypher("MATCH (n) /* x */ DELETE n").is_err());
    }

    #[test]
    fn sanitize_limit_handling() {
        assert_eq!(
            sanitize_cypher("MATCH (n) RETURN n;").unwrap(),
            "MATCH (n) RETURN n LIMIT 200"
        );
        // a trailing comment must not swallow the appended LIMIT
        let q = sanitize_cypher("MATCH (n) RETURN n // all").unwrap();
        assert!(q.ends_with("LIMIT 200") && !q.contains("//"), "{q}");
        // 'LIMIT' inside a string literal does not count as a limit
        assert!(
            sanitize_cypher("MATCH (n) WHERE n.title = 'LIMIT' RETURN n")
                .unwrap()
                .ends_with("LIMIT 200")
        );
    }

    #[test]
    fn sanitize_preserves_existing_limit() {
        let result = sanitize_cypher("MATCH (n) RETURN n LIMIT 10").unwrap();
        assert_eq!(result, "MATCH (n) RETURN n LIMIT 10");
    }

    #[test]
    fn clean_cypher_strips_markdown() {
        assert_eq!(
            clean_cypher("```cypher\nMATCH (n) RETURN n\n```"),
            "MATCH (n) RETURN n"
        );
        assert_eq!(clean_cypher("MATCH (n) RETURN n"), "MATCH (n) RETURN n");
        assert_eq!(clean_cypher("  MATCH (n) RETURN n  "), "MATCH (n) RETURN n");
    }
}
