use crate::error::AppResult;
use crate::models::search::SearchResult;
use pgvector::Vector;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn semantic_search(
    pool: &PgPool,
    vault_id: Uuid,
    vector: &Vector,
    limit: i64,
) -> AppResult<Vec<SearchResult>> {
    Ok(sqlx::query_as::<_, SearchResult>(
        r#"SELECT id as node_id, title, content_type,
           substring(content, 1, 200) as snippet,
           1 - (embedding <=> $1) as score
           FROM nodes
           WHERE vault_id = $2 AND embedding IS NOT NULL
           ORDER BY embedding <=> $1
           LIMIT $3"#,
    )
    .bind(vector)
    .bind(vault_id)
    .bind(limit)
    .fetch_all(pool)
    .await?)
}

pub async fn find_similar(
    pool: &PgPool,
    node_id: Uuid,
    limit: i64,
) -> AppResult<Vec<SearchResult>> {
    Ok(sqlx::query_as::<_, SearchResult>(
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
    .fetch_all(pool)
    .await?)
}
