use runestone_server::{build_router, ServerState, MAX_INVOKE_BODY_BYTES};
use runestone_core::db::{create_neo4j_graph, create_pg_pool, run_neo4j_init, run_pg_migrations};
use runestone_core::handlers::embeddings;
use runestone_core::{BackendContext, EmbeddingConfig, LlmConfig};
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "runestone_server=info,tower_http=info".into()),
        )
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://runestone:runestone@postgres:5432/runestone".to_string());
    let neo4j_uri =
        std::env::var("NEO4J_URL").unwrap_or_else(|_| "bolt://neo4j:7687".to_string());
    let neo4j_user = std::env::var("NEO4J_USER").unwrap_or_else(|_| "neo4j".to_string());
    let neo4j_password =
        std::env::var("NEO4J_PASSWORD").unwrap_or_else(|_| "runestone".to_string());
    let api_token = std::env::var("RUNESTONE_API_TOKEN").ok();
    let port: u16 = std::env::var("RUNESTONE_SERVER_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);

    info!("Connecting to PostgreSQL...");
    let pg = create_pg_pool(&database_url).await?;
    run_pg_migrations(&pg).await?;
    info!("PostgreSQL ready");

    info!("Connecting to Neo4j...");
    let neo4j = create_neo4j_graph(&neo4j_uri, &neo4j_user, &neo4j_password).await?;
    run_neo4j_init(&neo4j).await?;
    info!("Neo4j ready");

    let ctx = BackendContext {
        pg,
        neo4j,
        embed_config: EmbeddingConfig::default(),
        llm_config: LlmConfig::default(),
    };

    embeddings::spawn_embedding_worker(ctx.clone());

    let state = Arc::new(ServerState { ctx, api_token });
    let app = build_router(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!(
        "Runestone server listening on {} (max invoke body {} bytes)",
        addr, MAX_INVOKE_BODY_BYTES
    );

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
