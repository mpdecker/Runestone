use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "runestone_server=info,tower_http=info".into()),
        )
        .init();

    let static_dir = std::env::args()
        .nth(1)
        .filter(|a| !a.starts_with("--"))
        .or_else(|| std::env::var("STATIC_DIR").ok());

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://runestone:runestone@postgres:5432/runestone".to_string());
    let neo4j_uri =
        std::env::var("NEO4J_URL").unwrap_or_else(|_| "bolt://neo4j:7687".to_string());
    let neo4j_user = std::env::var("NEO4J_USER").unwrap_or_else(|_| "neo4j".to_string());
    let neo4j_password =
        std::env::var("NEO4J_PASSWORD").unwrap_or_else(|_| "runestone".to_string());
    let api_token = std::env::var("RUNESTONE_API_TOKEN").ok();
    match api_token.as_deref() {
        None | Some("") => tracing::warn!(
            "RUNESTONE_API_TOKEN is not set: every /api/invoke command (including run_cypher and              note deletion) is open to anyone who can reach this port. Set a token before exposing the server."
        ),
        Some("runestone-dev-token") | Some("change-me-in-production") => tracing::warn!(
            "RUNESTONE_API_TOKEN is a well-known placeholder value; set a unique secret before exposing the server."
        ),
        _ => {}
    }
    let port: u16 = std::env::var("RUNESTONE_SERVER_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);

    let pg = runestone_core::db::create_pg_pool(&database_url).await?;
    runestone_core::db::run_pg_migrations(&pg).await?;

    let neo4j =
        runestone_core::db::create_neo4j_graph(&neo4j_uri, &neo4j_user, &neo4j_password).await?;
    runestone_core::db::run_neo4j_init(&neo4j).await?;

    let embed_config = runestone_core::EmbeddingConfig::default();
    let llm_config = runestone_core::LlmConfig::default();

    let ctx = runestone_core::BackendContext {
        pg,
        neo4j,
        embed_config: embed_config.clone(),
        llm_config,
    };

    runestone_core::handlers::embeddings::spawn_embedding_worker(ctx.clone());

    let state = Arc::new(runestone_server::ServerState {
        ctx,
        api_token,
        static_dir,
    });

    let app = runestone_server::build_router(state);
    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("runestone-server listening on {}", addr);
    axum::serve(listener, app).await?;
    Ok(())
}
