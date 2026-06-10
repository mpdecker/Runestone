use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObsidianImportResult {
    pub nodes_created: i32,
    pub links_created: i32,
    pub files_scanned: i32,
}
