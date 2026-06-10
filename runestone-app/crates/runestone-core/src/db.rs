use neo4rs::*;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::sync::Arc;

pub async fn create_pg_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await
}

pub async fn create_neo4j_graph(uri: &str, user: &str, password: &str) -> Result<Arc<Graph>, neo4rs::Error> {
    let graph = Graph::new(uri, user, password).await?;
    Ok(Arc::new(graph))
}

pub async fn run_pg_migrations(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .map_err(|e| sqlx::Error::Migrate(Box::new(e)))
}

pub async fn run_neo4j_init(graph: &Graph) -> Result<(), neo4rs::Error> {
    let queries = vec![
        "CREATE INDEX node_pg_id IF NOT EXISTS FOR (n:Node) ON (n.pg_id);",
        "CREATE INDEX node_vault IF NOT EXISTS FOR (n:Node) ON (n.vault_id);",
        "CREATE INDEX node_type IF NOT EXISTS FOR (n:Node) ON (n.content_type);",
        "CREATE INDEX node_title IF NOT EXISTS FOR (n:Node) ON (n.title);",
        "CREATE INDEX tag_name IF NOT EXISTS FOR (t:Tag) ON (t.name);",
        "CREATE CONSTRAINT tag_name_unique IF NOT EXISTS FOR (t:Tag) REQUIRE t.name IS UNIQUE;",
    ];

    for q in queries {
        graph.run(neo4rs::query(q)).await?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_pg_pool_bad_url_fails() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(create_pg_pool("postgres://invalid:invalid@localhost:1/db"));
        assert!(result.is_err());
    }
}
