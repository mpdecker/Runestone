use crate::embedding::EmbeddingConfig;
use crate::llm::LlmConfig;
use neo4rs::Graph;
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct BackendContext {
    pub pg: PgPool,
    pub neo4j: Arc<Graph>,
    pub embed_config: EmbeddingConfig,
    pub llm_config: LlmConfig,
}
