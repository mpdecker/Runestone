use crate::context::BackendContext;
use crate::embedding::generate_embedding;
use crate::models::node::Node;
use crate::models::search::{SearchQuery, SearchResult, SearchResults};
use uuid::Uuid;

const MAX_REGEX_PATTERN_LEN: usize = 200;
const RRF_K: f64 = 60.0;

/// Reciprocal Rank Fusion: merges ranked result lists by node_id.
pub fn rrf_fusion(
    ranked_lists: &[Vec<SearchResult>],
    limit: usize,
) -> Vec<SearchResult> {
    let mut scores: std::collections::HashMap<uuid::Uuid, f64> =
        std::collections::HashMap::new();
    let mut best: std::collections::HashMap<uuid::Uuid, SearchResult> =
        std::collections::HashMap::new();

    for list in ranked_lists {
        for (rank, result) in list.iter().enumerate() {
            let rrf_score = 1.0 / (RRF_K + (rank as f64) + 1.0);
            *scores.entry(result.node_id).or_insert(0.0) += rrf_score;
            best.entry(result.node_id).or_insert_with(|| result.clone());
        }
    }

    let mut fused: Vec<(uuid::Uuid, f64)> = scores.into_iter().collect();
    fused.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    fused.truncate(limit);

    fused
        .into_iter()
        .filter_map(|(id, score)| {
            best.get(&id).map(|r| SearchResult {
                score,
                ..r.clone()
            })
        })
        .collect()
}

pub fn parse_boolean_query(raw: &str) -> String {
    raw.replace(" AND ", " & ")
        .replace(" and ", " & ")
        .replace(" OR ", " | ")
        .replace(" or ", " | ")
        .replace(" NOT ", " !")
        .replace(" not ", " !")
}

pub async fn semantic_search(
    ctx: &BackendContext,
    query: SearchQuery,
) -> Result<Vec<SearchResult>, String> {
    let embedding = generate_embedding(&query.query, &ctx.embed_config).await?;
    let vector = pgvector::Vector::from(embedding);
    let limit = query.limit.unwrap_or(20);

    let results = sqlx::query_as::<_, SearchResult>(
        r#"SELECT id as node_id, title, content_type,
           substring(content, 1, 200) as snippet,
           1 - (embedding <=> $1) as score
           FROM nodes
           WHERE vault_id = $2 AND embedding IS NOT NULL
           ORDER BY embedding <=> $1
           LIMIT $3"#,
    )
    .bind(&vector)
    .bind(query.vault_id)
    .bind(limit)
    .fetch_all(&ctx.pg)
    .await
    .map_err(|e| format!("Semantic search failed: {}", e))?;

    Ok(results)
}

pub async fn find_similar(
    ctx: &BackendContext,
    node_id: Uuid,
    limit: Option<i64>,
) -> Result<Vec<SearchResult>, String> {
    let limit = limit.unwrap_or(10);

    let results = sqlx::query_as::<_, SearchResult>(
        r#"SELECT n2.id as node_id, n2.title, n2.content_type,
           substring(n2.content, 1, 200) as snippet,
           1 - (n1.embedding <=> n2.embedding) as score
           FROM nodes n1, nodes n2
           WHERE n1.id = $1
             AND n2.id != $1
             AND n2.embedding IS NOT NULL
             AND n1.embedding IS NOT NULL
           ORDER BY n1.embedding <=> n2.embedding
           LIMIT $2"#,
    )
    .bind(node_id)
    .bind(limit)
    .fetch_all(&ctx.pg)
    .await
    .map_err(|e| format!("Find similar failed: {}", e))?;

    Ok(results)
}

pub async fn hybrid_search(
    ctx: &BackendContext,
    query: SearchQuery,
) -> Result<SearchResults, String> {
    let limit = query.limit.unwrap_or(20);
    let vault_id = query.vault_id;

    let vector_results: Vec<SearchResult> =
        match generate_embedding(&query.query, &ctx.embed_config).await {
            Ok(embedding) => {
                let vector = pgvector::Vector::from(embedding);
                sqlx::query_as::<_, SearchResult>(
                    r#"SELECT id as node_id, title, content_type,
                   substring(content, 1, 200) as snippet,
                   1 - (embedding <=> $1) as score
                   FROM nodes
                   WHERE vault_id = $2 AND embedding IS NOT NULL
                   ORDER BY embedding <=> $1
                   LIMIT $3"#,
                )
                .bind(&vector)
                .bind(vault_id)
                .bind(limit)
                .fetch_all(&ctx.pg)
                .await
                .unwrap_or_default()
            }
            Err(_) => Vec::new(),
        };

    let fts_results: Vec<SearchResult> = if query.include_fts.unwrap_or(true) {
        sqlx::query_as::<_, SearchResult>(
            r#"SELECT id as node_id, title, content_type,
               substring(content, 1, 200) as snippet,
               ts_rank_cd(to_tsvector('english', coalesce(title,'') || ' ' || coalesce(content,'')), plainto_tsquery('english', $1)) as score
               FROM nodes
               WHERE vault_id = $2
                 AND to_tsvector('english', coalesce(title,'') || ' ' || coalesce(content,'')) @@ plainto_tsquery('english', $1)
               ORDER BY score DESC
               LIMIT $3"#,
        )
        .bind(&query.query)
        .bind(vault_id)
        .bind(limit)
        .fetch_all(&ctx.pg)
        .await
        .unwrap_or_default()
    } else {
        Vec::new()
    };

    // TODO: optional SEARCH_RERANK env for cross-encoder reranking of fused results
    let combined = rrf_fusion(
        &[vector_results.clone(), fts_results.clone()],
        limit as usize,
    );

    Ok(SearchResults {
        vector_results,
        fts_results,
        combined,
    })
}

pub async fn boolean_search(
    ctx: &BackendContext,
    query: SearchQuery,
) -> Result<Vec<SearchResult>, String> {
    let ts_query = parse_boolean_query(&query.query);
    let limit = query.limit.unwrap_or(20);

    let results = sqlx::query_as::<_, SearchResult>(
        r#"SELECT id as node_id, title, content_type,
           ts_headline('english', content, to_tsquery('english', $1), 'MaxWords=30, MinWords=10') as snippet,
           ts_rank(to_tsvector('english', coalesce(content, '')), to_tsquery('english', $1)) as score
           FROM nodes
           WHERE vault_id = $2
             AND to_tsvector('english', coalesce(content, '')) @@ to_tsquery('english', $1)
           ORDER BY score DESC
           LIMIT $3"#,
    )
    .bind(&ts_query)
    .bind(query.vault_id)
    .bind(limit)
    .fetch_all(&ctx.pg)
    .await
    .map_err(|e| format!("Boolean search failed: {}", e))?;

    Ok(results)
}

pub async fn regex_search(
    ctx: &BackendContext,
    vault_id: Uuid,
    pattern: String,
    case_sensitive: Option<bool>,
    limit: Option<i64>,
) -> Result<Vec<SearchResult>, String> {
    if pattern.len() > MAX_REGEX_PATTERN_LEN {
        return Err(format!(
            "Regex pattern exceeds maximum length of {} characters",
            MAX_REGEX_PATTERN_LEN
        ));
    }
    if regex::RegexBuilder::new(&pattern)
        .case_insensitive(!case_sensitive.unwrap_or(false))
        .build()
        .is_err()
    {
        return Err("Invalid regex pattern".to_string());
    }

    let case = case_sensitive.unwrap_or(false);
    let limit = limit.unwrap_or(20).min(100);

    let results = if case {
        sqlx::query_as::<_, SearchResult>(
            r#"SELECT id as node_id, title, content_type,
               substring(content, 1, 200) as snippet,
               1.0 as score
               FROM nodes
               WHERE vault_id = $1 AND content ~ $2
               ORDER BY title
               LIMIT $3"#,
        )
        .bind(vault_id)
        .bind(&pattern)
        .bind(limit)
        .fetch_all(&ctx.pg)
        .await
        .map_err(|e| format!("Regex search failed: {}", e))?
    } else {
        sqlx::query_as::<_, SearchResult>(
            r#"SELECT id as node_id, title, content_type,
               substring(content, 1, 200) as snippet,
               1.0 as score
               FROM nodes
               WHERE vault_id = $1 AND content ~* $2
               ORDER BY title
               LIMIT $3"#,
        )
        .bind(vault_id)
        .bind(&pattern)
        .bind(limit)
        .fetch_all(&ctx.pg)
        .await
        .map_err(|e| format!("Regex search failed: {}", e))?
    };

    Ok(results)
}

pub async fn get_node_by_alias(
    ctx: &BackendContext,
    vault_id: Uuid,
    alias: String,
) -> Result<Option<Node>, String> {
    let nodes = sqlx::query_as::<_, Node>(
        r#"SELECT id, vault_id, title, content, content_type, file_path, metadata, word_count, created_at, updated_at
           FROM nodes
           WHERE vault_id = $1 AND metadata->'aliases' ? $2
           LIMIT 1"#,
    )
    .bind(vault_id)
    .bind(&alias)
    .fetch_optional(&ctx.pg)
    .await
    .map_err(|e| format!("Alias query failed: {}", e))?;

    Ok(nodes)
}

pub async fn add_alias(
    ctx: &BackendContext,
    node_id: Uuid,
    alias: String,
) -> Result<serde_json::Value, String> {
    let row = sqlx::query_as::<_, (Option<serde_json::Value>,)>(
        "SELECT metadata FROM nodes WHERE id = $1",
    )
    .bind(node_id)
    .fetch_one(&ctx.pg)
    .await
    .map_err(|e| format!("Node not found: {}", e))?;

    let mut meta = row.0.unwrap_or(serde_json::json!({}));
    if let Some(aliases) = meta.get_mut("aliases") {
        if let Some(arr) = aliases.as_array_mut() {
            let alias_val = serde_json::json!(alias);
            if !arr.contains(&alias_val) {
                arr.push(alias_val);
            }
        }
    } else {
        meta["aliases"] = serde_json::json!([alias]);
    }

    sqlx::query("UPDATE nodes SET metadata = $2, updated_at = NOW() WHERE id = $1")
        .bind(node_id)
        .bind(&meta)
        .execute(&ctx.pg)
        .await
        .map_err(|e| format!("Failed to update alias: {}", e))?;

    Ok(meta["aliases"].clone())
}

pub async fn remove_alias(
    ctx: &BackendContext,
    node_id: Uuid,
    alias: String,
) -> Result<serde_json::Value, String> {
    let row = sqlx::query_as::<_, (Option<serde_json::Value>,)>(
        "SELECT metadata FROM nodes WHERE id = $1",
    )
    .bind(node_id)
    .fetch_one(&ctx.pg)
    .await
    .map_err(|e| format!("Node not found: {}", e))?;

    let mut meta = row.0.unwrap_or(serde_json::json!({}));
    if let Some(aliases) = meta.get_mut("aliases") {
        if let Some(arr) = aliases.as_array_mut() {
            arr.retain(|v| v.as_str() != Some(&alias));
        }
    }

    sqlx::query("UPDATE nodes SET metadata = $2, updated_at = NOW() WHERE id = $1")
        .bind(node_id)
        .bind(&meta)
        .execute(&ctx.pg)
        .await
        .map_err(|e| format!("Failed to remove alias: {}", e))?;

    Ok(meta["aliases"].clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_boolean_query_replaces_operators() {
        assert_eq!(parse_boolean_query("foo AND bar"), "foo & bar");
        assert_eq!(parse_boolean_query("foo OR bar NOT baz"), "foo | bar !baz");
    }

    fn make_result(id: uuid::Uuid, title: &str, score: f64) -> SearchResult {
        SearchResult {
            node_id: id,
            title: title.to_string(),
            content_type: "note".to_string(),
            snippet: "snippet".to_string(),
            score,
        }
    }

    #[test]
    fn rrf_fusion_prefers_items_in_both_lists() {
        let a = uuid::Uuid::new_v4();
        let b = uuid::Uuid::new_v4();
        let list1 = vec![make_result(a, "A", 0.9), make_result(b, "B", 0.5)];
        let list2 = vec![make_result(a, "A", 0.8)];
        let fused = rrf_fusion(&[list1, list2], 10);
        assert_eq!(fused[0].node_id, a);
    }

    #[test]
    fn rrf_fusion_respects_limit() {
        let ids: Vec<_> = (0..5).map(|_| uuid::Uuid::new_v4()).collect();
        let list = ids
            .iter()
            .map(|id| make_result(*id, "X", 1.0_f64))
            .collect::<Vec<_>>();
        let fused = rrf_fusion(&[list], 3);
        assert_eq!(fused.len(), 3);
    }
}
