use crate::db::{create_neo4j_graph, create_pg_pool};
use crate::embedding::EmbeddingConfig;
use crate::llm::LlmConfig;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::{Arc, Mutex};
use tauri::Manager;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConnectionMode {
    Local,
    Remote {
        api_url: String,
        auth_token: Option<String>,
    },
}

pub struct AppState {
    pub connection_mode: Mutex<ConnectionMode>,
    pg: Option<PgPool>,
    neo4j: Option<Arc<neo4rs::Graph>>,
    pub embed_config: EmbeddingConfig,
    pub llm_config: LlmConfig,
}

impl AppState {
    pub fn init(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
        let handle = app.handle();

        if cfg!(debug_assertions) {
            handle.plugin(
                tauri_plugin_log::Builder::default()
                    .level(log::LevelFilter::Info)
                    .build(),
            )?;
        }

        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://runestone:runestone@localhost:5432/runestone".to_string());
        let neo4j_uri = std::env::var("NEO4J_URL")
            .unwrap_or_else(|_| "bolt://localhost:7687".to_string());
        let neo4j_user = std::env::var("NEO4J_USER")
            .unwrap_or_else(|_| "neo4j".to_string());
        let neo4j_password = std::env::var("NEO4J_PASSWORD")
            .unwrap_or_else(|_| "runestone".to_string());

        let is_mobile = cfg!(target_os = "ios") || cfg!(target_os = "android");

        let (pg_pool, neo4j_graph, mode) = if is_mobile {
            (None, None, ConnectionMode::Remote {
                api_url: String::new(),
                auth_token: None,
            })
        } else {
            let pg_result = tauri::async_runtime::block_on(create_pg_pool(&database_url));
            let neo4j_result = tauri::async_runtime::block_on(create_neo4j_graph(
                &neo4j_uri,
                &neo4j_user,
                &neo4j_password,
            ));

            match (pg_result, neo4j_result) {
                (Ok(pg), Ok(neo4j)) => {
                    log::info!("Local database connections established");
                    (Some(pg), Some(neo4j), ConnectionMode::Local)
                }
                (pg_err, neo4j_err) => {
                    log::warn!(
                        "Database connections not available (PG: {:?}, Neo4j: {:?}). Starting in Remote mode.",
                        pg_err.as_ref().err(),
                        neo4j_err.as_ref().err()
                    );
                    (None, None, ConnectionMode::Remote {
                        api_url: String::new(),
                        auth_token: None,
                    })
                }
            }
        };

        let state = AppState {
            connection_mode: Mutex::new(mode),
            pg: pg_pool,
            neo4j: neo4j_graph,
            embed_config: EmbeddingConfig::default(),
            llm_config: LlmConfig::default(),
        };

        app.manage(state);

        Ok(())
    }

    pub fn pg(&self) -> Result<&PgPool, String> {
        self.pg.as_ref().ok_or_else(|| {
            "Local database not available. Connect to a remote Runestone server to use this feature.".to_string()
        })
    }

    pub fn neo4j(&self) -> Result<&Arc<neo4rs::Graph>, String> {
        self.neo4j.as_ref().ok_or_else(|| {
            "Local graph database not available. Connect to a remote Runestone server to use this feature.".to_string()
        })
    }

    pub fn has_pg(&self) -> bool {
        self.pg.is_some()
    }

    pub fn has_neo4j(&self) -> bool {
        self.neo4j.is_some()
    }

    pub fn set_remote_config(&self, api_url: String, auth_token: Option<String>) {
        if let Ok(mut mode) = self.connection_mode.lock() {
            *mode = ConnectionMode::Remote { api_url, auth_token };
        }
    }

    pub fn get_remote_config(&self) -> Option<(String, Option<String>)> {
        if let Ok(mode) = self.connection_mode.lock() {
            if let ConnectionMode::Remote { api_url, auth_token } = mode.clone() {
                return Some((api_url, auth_token));
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_mode_serialization() {
        let local = ConnectionMode::Local;
        let json = serde_json::to_string(&local).unwrap();
        assert!(json.contains("Local"));

        let deserialized: ConnectionMode = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, ConnectionMode::Local);
    }

    #[test]
    fn test_connection_mode_remote_serialization() {
        let remote = ConnectionMode::Remote {
            api_url: "https://server.com".to_string(),
            auth_token: Some("token123".to_string()),
        };
        let json = serde_json::to_string(&remote).unwrap();
        assert!(json.contains("https://server.com"));
        assert!(json.contains("token123"));

        let deserialized: ConnectionMode = serde_json::from_str(&json).unwrap();
        match deserialized {
            ConnectionMode::Remote { api_url, auth_token } => {
                assert_eq!(api_url, "https://server.com");
                assert_eq!(auth_token, Some("token123".to_string()));
            }
            _ => panic!("Expected Remote mode"),
        }
    }

    #[test]
    fn test_get_remote_config_returns_none_when_local() {
        let state = AppState {
            connection_mode: Mutex::new(ConnectionMode::Local),
            pg: None,
            neo4j: None,
            embed_config: EmbeddingConfig::default(),
            llm_config: LlmConfig::default(),
        };
        assert!(state.get_remote_config().is_none());
    }

    #[test]
    fn test_get_remote_config_returns_url_when_remote() {
        let state = AppState {
            connection_mode: Mutex::new(ConnectionMode::Remote {
                api_url: "http://example.com".to_string(),
                auth_token: Some("bearer".to_string()),
            }),
            pg: None,
            neo4j: None,
            embed_config: EmbeddingConfig::default(),
            llm_config: LlmConfig::default(),
        };
        let config = state.get_remote_config().unwrap();
        assert_eq!(config.0, "http://example.com");
        assert_eq!(config.1, Some("bearer".to_string()));
    }

    #[test]
    fn test_set_remote_config_updates_mode() {
        let state = AppState {
            connection_mode: Mutex::new(ConnectionMode::Local),
            pg: None,
            neo4j: None,
            embed_config: EmbeddingConfig::default(),
            llm_config: LlmConfig::default(),
        };
        state.set_remote_config("https://new.com".to_string(), None);
        let config = state.get_remote_config().unwrap();
        assert_eq!(config.0, "https://new.com");
        assert!(config.1.is_none());
    }

    #[test]
    fn test_has_pg_returns_false_when_none() {
        let state = AppState {
            connection_mode: Mutex::new(ConnectionMode::Local),
            pg: None,
            neo4j: None,
            embed_config: EmbeddingConfig::default(),
            llm_config: LlmConfig::default(),
        };
        assert!(!state.has_pg());
    }

    #[test]
    fn test_pg_returns_err_when_none() {
        let state = AppState {
            connection_mode: Mutex::new(ConnectionMode::Local),
            pg: None,
            neo4j: None,
            embed_config: EmbeddingConfig::default(),
            llm_config: LlmConfig::default(),
        };
        assert!(state.pg().is_err());
    }

    #[test]
    fn test_neo4j_returns_err_when_none() {
        let state = AppState {
            connection_mode: Mutex::new(ConnectionMode::Local),
            pg: None,
            neo4j: None,
            embed_config: EmbeddingConfig::default(),
            llm_config: LlmConfig::default(),
        };
        assert!(state.neo4j().is_err());
    }
}
