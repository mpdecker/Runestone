use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error, Serialize)]
#[serde(tag = "code", content = "message")]
pub enum AppError {
    #[error("Database unavailable: {0}")]
    DatabaseUnavailable(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Desktop-only command: {0}")]
    DesktopOnly(String),
    #[error("PostgreSQL error: {0}")]
    Postgres(String),
    #[error("Neo4j error: {0}")]
    Neo4j(String),
    #[error("IO error: {0}")]
    Io(String),
    #[error("{0}")]
    Other(String),
}

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        AppError::Postgres(e.to_string())
    }
}

impl From<neo4rs::Error> for AppError {
    fn from(e: neo4rs::Error) -> Self {
        AppError::Neo4j(e.to_string())
    }
}

impl From<neo4rs::DeError> for AppError {
    fn from(e: neo4rs::DeError) -> Self {
        AppError::Neo4j(e.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::Io(e.to_string())
    }
}

impl From<AppError> for String {
    fn from(e: AppError) -> String {
        e.to_string()
    }
}

pub type AppResult<T> = Result<T, AppError>;
