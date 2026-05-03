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
    sqlx::query(
        r#"
        CREATE EXTENSION IF NOT EXISTS vector;
        CREATE EXTENSION IF NOT EXISTS pg_trgm;
        CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS vaults (
            id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
            name        TEXT NOT NULL,
            root_path   TEXT NOT NULL,
            created_at  TIMESTAMPTZ DEFAULT NOW(),
            updated_at  TIMESTAMPTZ DEFAULT NOW()
        );
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS nodes (
            id            UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
            vault_id      UUID NOT NULL REFERENCES vaults(id) ON DELETE CASCADE,
            title         TEXT NOT NULL,
            content       TEXT NOT NULL DEFAULT '',
            content_type  TEXT NOT NULL DEFAULT 'note',
            embedding     vector(1536),
            file_path     TEXT,
            metadata      JSONB DEFAULT '{}',
            word_count    INTEGER DEFAULT 0,
            created_at    TIMESTAMPTZ DEFAULT NOW(),
            updated_at    TIMESTAMPTZ DEFAULT NOW()
        );
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_nodes_vault ON nodes(vault_id);")
        .execute(pool)
        .await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_nodes_type ON nodes(content_type);")
        .execute(pool)
        .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_nodes_title_trgm ON nodes USING gin (title gin_trgm_ops);",
    )
    .execute(pool)
    .await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_nodes_fts ON nodes USING gin (to_tsvector('english', coalesce(title,'') || ' ' || coalesce(content,'')));")
        .execute(pool)
        .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS document_chunks (
            id               UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
            document_node_id UUID NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
            chunk_index      INTEGER NOT NULL,
            content          TEXT NOT NULL,
            embedding        vector(1536),
            token_count      INTEGER DEFAULT 0,
            metadata         JSONB DEFAULT '{}',
            created_at       TIMESTAMPTZ DEFAULT NOW()
        );
        "#,
    )
    .execute(pool)
    .await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_chunks_doc ON document_chunks(document_node_id);")
        .execute(pool)
        .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS wiki_links (
            id               UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
            source_node_id   UUID NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
            target_title     TEXT NOT NULL,
            resolved_node_id UUID REFERENCES nodes(id),
            context          TEXT,
            created_at       TIMESTAMPTZ DEFAULT NOW()
        );
        "#,
    )
    .execute(pool)
    .await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_wikilinks_source ON wiki_links(source_node_id);")
        .execute(pool)
        .await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_wikilinks_unresolved ON wiki_links(resolved_node_id) WHERE resolved_node_id IS NULL;")
        .execute(pool)
        .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS node_versions (
            id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
            node_id         UUID NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
            version_number  INTEGER NOT NULL,
            title           TEXT NOT NULL,
            content         TEXT NOT NULL,
            word_count      INTEGER NOT NULL DEFAULT 0,
            created_at      TIMESTAMPTZ DEFAULT NOW()
        );
        "#,
    )
    .execute(pool)
    .await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_versions_node ON node_versions(node_id, version_number DESC);")
        .execute(pool)
        .await?;

    Ok(())
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
