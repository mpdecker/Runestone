use crate::embedding::generate_embedding;
use crate::models::search::{SearchQuery, SearchResult, SearchResults};
use crate::state::AppState;
use uuid::Uuid;

#[tauri::command]
pub async fn semantic_search(
    state: tauri::State<'_, AppState>,
    query: SearchQuery,
) -> Result<Vec<SearchResult>, String> {
    let embed_config = &state.embed_config;
    let embed_text = &query.query;
    let embedding = generate_embedding(embed_text, embed_config).await?;

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
    .fetch_all(state.pg()?)
    .await
    .map_err(|e| format!("Semantic search failed: {}", e))?;

    Ok(results)
}

#[tauri::command]
pub async fn find_similar(
    state: tauri::State<'_, AppState>,
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
    .fetch_all(state.pg()?)
    .await
    .map_err(|e| format!("Find similar failed: {}", e))?;

    Ok(results)
}

#[tauri::command]
pub async fn hybrid_search(
    state: tauri::State<'_, AppState>,
    query: SearchQuery,
) -> Result<SearchResults, String> {
    let limit = query.limit.unwrap_or(20);
    let vault_id = query.vault_id;

    let vector_results: Vec<SearchResult> = match generate_embedding(&query.query, &state.embed_config).await {
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
            .fetch_all(state.pg()?)
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
        .fetch_all(state.pg()?)
        .await
        .unwrap_or_default()
    } else {
        Vec::new()
    };

    let mut combined = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for r in &vector_results {
        if seen.insert(r.node_id) {
            combined.push(r.clone());
        }
    }
    for r in &fts_results {
        if seen.insert(r.node_id) {
            combined.push(r.clone());
        }
    }
    combined.truncate(limit as usize);

    Ok(SearchResults {
        vector_results,
        fts_results,
        combined,
    })
}

fn parse_boolean_query(raw: &str) -> String {
    let processed = raw
        .replace(" AND ", " & ")
        .replace(" and ", " & ")
        .replace(" OR ", " | ")
        .replace(" or ", " | ")
        .replace(" NOT ", " !")
        .replace(" not ", " !");

    processed
}

#[tauri::command]
pub async fn boolean_search(
    state: tauri::State<'_, AppState>,
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
    .fetch_all(state.pg()?)
    .await
    .map_err(|e| format!("Boolean search failed: {}", e))?;

    Ok(results)
}

#[tauri::command]
pub async fn regex_search(
    state: tauri::State<'_, AppState>,
    vault_id: Uuid,
    pattern: String,
    case_sensitive: Option<bool>,
    limit: Option<i64>,
) -> Result<Vec<SearchResult>, String> {
    let case = case_sensitive.unwrap_or(false);
    let limit = limit.unwrap_or(20);

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
        .fetch_all(state.pg()?)
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
        .fetch_all(state.pg()?)
        .await
        .map_err(|e| format!("Regex search failed: {}", e))?
    };

    Ok(results)
}

#[tauri::command]
pub async fn get_node_by_alias(
    state: tauri::State<'_, AppState>,
    vault_id: Uuid,
    alias: String,
) -> Result<Option<crate::models::node::Node>, String> {
    let nodes = sqlx::query_as::<_, crate::models::node::Node>(
        r#"SELECT id, vault_id, title, content, content_type, file_path, metadata, word_count, created_at, updated_at
           FROM nodes
           WHERE vault_id = $1 AND metadata->'aliases' ? $2
           LIMIT 1"#,
    )
    .bind(vault_id)
    .bind(&alias)
    .fetch_optional(state.pg()?)
    .await
    .map_err(|e| format!("Alias query failed: {}", e))?;

    Ok(nodes)
}

#[tauri::command]
pub async fn add_alias(
    state: tauri::State<'_, AppState>,
    node_id: Uuid,
    alias: String,
) -> Result<serde_json::Value, String> {
    let row = sqlx::query_as::<_, (Option<serde_json::Value>,)>(
        "SELECT metadata FROM nodes WHERE id = $1",
    )
    .bind(node_id)
    .fetch_one(state.pg()?)
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
        .execute(state.pg()?)
        .await
        .map_err(|e| format!("Failed to update alias: {}", e))?;

    Ok(meta["aliases"].clone())
}

#[tauri::command]
pub async fn remove_alias(
    state: tauri::State<'_, AppState>,
    node_id: Uuid,
    alias: String,
) -> Result<serde_json::Value, String> {
    let row = sqlx::query_as::<_, (Option<serde_json::Value>,)>(
        "SELECT metadata FROM nodes WHERE id = $1",
    )
    .bind(node_id)
    .fetch_one(state.pg()?)
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
        .execute(state.pg()?)
        .await
        .map_err(|e| format!("Failed to remove alias: {}", e))?;

    Ok(meta["aliases"].clone())
}
