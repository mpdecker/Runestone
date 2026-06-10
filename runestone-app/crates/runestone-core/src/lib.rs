pub mod context;
pub mod db;
pub mod dispatch;
pub mod document;
pub mod embedding;
pub mod error;
pub mod handlers;
pub mod llm;
pub mod models;
pub mod path_guard;
pub mod repositories;
pub mod services;
pub mod util;

pub use context::BackendContext;
pub use embedding::EmbeddingConfig;
pub use error::{AppError, AppResult};
pub use llm::LlmConfig;
