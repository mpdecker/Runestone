use runestone_core::db::{create_neo4j_graph, create_pg_pool, run_neo4j_init, run_pg_migrations};
use runestone_core::handlers::embeddings;
use runestone_core::embedding::EmbeddingConfig;
use runestone_core::llm::LlmConfig;
use runestone_core::BackendContext;
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
    pg: Mutex<Option<PgPool>>,
    neo4j: Mutex<Option<Arc<neo4rs::Graph>>>,
    pub is_remote_connected: Mutex<bool>,
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

        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://runestone:runestone@localhost:5442/runestone".to_string()
        });
        let neo4j_uri =
            std::env::var("NEO4J_URL").unwrap_or_else(|_| "bolt://localhost:7688".to_string());
        let neo4j_user = std::env::var("NEO4J_USER").unwrap_or_else(|_| "neo4j".to_string());
        let neo4j_password =
            std::env::var("NEO4J_PASSWORD").unwrap_or_else(|_| "runestone".to_string());

        let is_mobile = cfg!(target_os = "ios") || cfg!(target_os = "android");

        let (pg_pool, neo4j_graph, mode) = if is_mobile {
            (
                None,
                None,
                ConnectionMode::Remote {
                    api_url: String::new(),
                    auth_token: None,
                },
            )
        } else {
            let pg_result = tauri::async_runtime::block_on(create_pg_pool(&database_url));
            let neo4j_result = tauri::async_runtime::block_on(create_neo4j_graph(
                &neo4j_uri,
                &neo4j_user,
                &neo4j_password,
            ));

            match (pg_result, neo4j_result) {
                (Ok(pg), Ok(neo4j)) => {
                    if let Err(e) = tauri::async_runtime::block_on(run_pg_migrations(&pg)) {
                        log::warn!("PostgreSQL migration failed: {}", e);
                    }
                    if let Err(e) = tauri::async_runtime::block_on(run_neo4j_init(&neo4j)) {
                        log::warn!("Neo4j initialization failed: {}", e);
                    }
                    log::info!("Local database connections established");
                    (Some(pg), Some(neo4j), ConnectionMode::Local)
                }
                (pg_err, neo4j_err) => {
                    log::warn!(
                        "Database connections not available (PG: {:?}, Neo4j: {:?}). Starting in Remote mode.",
                        pg_err.as_ref().err(),
                        neo4j_err.as_ref().err()
                    );
                    (
                        None,
                        None,
                        ConnectionMode::Remote {
                            api_url: String::new(),
                            auth_token: None,
                        },
                    )
                }
            }
        };

        let state = AppState {
            connection_mode: Mutex::new(mode),
            pg: Mutex::new(pg_pool),
            neo4j: Mutex::new(neo4j_graph),
            is_remote_connected: Mutex::new(false),
            embed_config: EmbeddingConfig::default(),
            llm_config: LlmConfig::default(),
        };

        app.manage(state);

        if let Some(state) = app.try_state::<AppState>() {
            if state.has_local_pools() {
                if let Ok(ctx) = state.backend_context() {
                    embeddings::spawn_embedding_worker(ctx);
                }
            }
        }

        Ok(())
    }

    pub fn is_local(&self) -> bool {
        self.connection_mode
            .lock()
            .map(|mode| matches!(*mode, ConnectionMode::Local))
            .unwrap_or(false)
    }

    pub fn is_remote_configured(&self) -> bool {
        self.get_remote_config()
            .map(|(url, _)| !url.is_empty())
            .unwrap_or(false)
    }

    pub fn has_local_pools(&self) -> bool {
        self.has_pg() && self.has_neo4j()
    }

    pub fn clear_local_pools(&self) {
        if let Ok(mut pg) = self.pg.lock() {
            *pg = None;
        }
        if let Ok(mut neo4j) = self.neo4j.lock() {
            *neo4j = None;
        }
    }

    pub fn set_remote_connected(&self, connected: bool) {
        if let Ok(mut flag) = self.is_remote_connected.lock() {
            *flag = connected;
        }
    }

    pub fn remote_connected(&self) -> bool {
        self.is_remote_connected.lock().map(|f| *f).unwrap_or(false)
    }

    pub fn backend_context(&self) -> Result<BackendContext, String> {
        Ok(BackendContext {
            pg: self.pg()?.clone(),
            neo4j: self.neo4j()?.clone(),
            embed_config: self.embed_config.clone(),
            llm_config: self.llm_config.clone(),
        })
    }

    pub fn pg(&self) -> Result<PgPool, String> {
        self.pg
            .lock()
            .map_err(|e| e.to_string())?
            .clone()
            .ok_or_else(|| {
                "Local database not available. Connect to a remote Runestone server to use this feature.".to_string()
            })
    }

    pub fn neo4j(&self) -> Result<Arc<neo4rs::Graph>, String> {
        self.neo4j
            .lock()
            .map_err(|e| e.to_string())?
            .clone()
            .ok_or_else(|| {
                "Local graph database not available. Connect to a remote Runestone server to use this feature.".to_string()
            })
    }

    pub fn has_pg(&self) -> bool {
        self.pg.lock().map(|p| p.is_some()).unwrap_or(false)
    }

    pub fn has_neo4j(&self) -> bool {
        self.neo4j.lock().map(|n| n.is_some()).unwrap_or(false)
    }

    pub fn set_remote_config(&self, api_url: String, auth_token: Option<String>) {
        if let Ok(mut mode) = self.connection_mode.lock() {
            *mode = ConnectionMode::Remote { api_url, auth_token };
        }
        self.set_remote_connected(false);
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

    fn test_state(mode: ConnectionMode) -> AppState {
        AppState {
            connection_mode: Mutex::new(mode),
            pg: Mutex::new(None),
            neo4j: Mutex::new(None),
            is_remote_connected: Mutex::new(false),
            embed_config: EmbeddingConfig::default(),
            llm_config: LlmConfig::default(),
        }
    }

    #[test]
    fn test_connection_mode_serialization() {
        let local = ConnectionMode::Local;
        let json = serde_json::to_string(&local).unwrap();
        assert!(json.contains("Local"));

        let deserialized: ConnectionMode = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, ConnectionMode::Local);
    }

    #[test]
    fn test_is_local() {
        let state = test_state(ConnectionMode::Local);
        assert!(state.is_local());

        let state = test_state(ConnectionMode::Remote {
            api_url: "http://x".to_string(),
            auth_token: None,
        });
        assert!(!state.is_local());
    }

    #[test]
    fn test_get_remote_config_returns_none_when_local() {
        let state = test_state(ConnectionMode::Local);
        assert!(state.get_remote_config().is_none());
    }

    #[test]
    fn test_set_remote_config_updates_mode() {
        let state = test_state(ConnectionMode::Local);
        state.set_remote_config("https://new.com".to_string(), None);
        let config = state.get_remote_config().unwrap();
        assert_eq!(config.0, "https://new.com");
        assert!(config.1.is_none());
    }

    #[test]
    fn test_clear_local_pools() {
        let state = test_state(ConnectionMode::Local);
        state.clear_local_pools();
        assert!(!state.has_pg());
    }

    #[test]
    fn test_remote_connected_flag() {
        let state = test_state(ConnectionMode::Remote {
            api_url: "http://x".to_string(),
            auth_token: None,
        });
        assert!(!state.remote_connected());
        state.set_remote_connected(true);
        assert!(state.remote_connected());
    }
}
