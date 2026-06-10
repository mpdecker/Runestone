pub mod vault;
pub mod node;
pub mod graph;
pub mod search;
pub mod extraction;
pub mod chat;
pub mod obsidian;
pub mod tag;
pub mod properties;
pub mod version;

#[allow(unused_imports)]
pub use vault::{CreateVaultRequest, Vault};
#[allow(unused_imports)]
pub use node::{CreateNodeRequest, ListNodesRequest, Node, NodeIdRow, NodeListItem, ScanVaultResult, UpdateNodeRequest};
#[allow(unused_imports)]
pub use graph::{Backlink, GraphData, GraphEdge, GraphNode, WikiLinkRow};
#[allow(unused_imports)]
pub use search::{SearchQuery, SearchResult, SearchResults};
#[allow(unused_imports)]
pub use extraction::{ExtractionNode, PendingExtraction};
#[allow(unused_imports)]
pub use chat::{ChatMessage, ChatRequest, ChatResponse, Citation, TagSuggestion};
#[allow(unused_imports)]
pub use obsidian::ObsidianImportResult;
#[allow(unused_imports)]
pub use properties::{NodeProperty, PropertiesResponse, SetPropertyRequest};
#[allow(unused_imports)]
pub use tag::{AddTagsRequest, RemoveTagRequest, TagInfo, TagsResponse};
#[allow(unused_imports)]
pub use version::NodeVersion;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_result_deserialization() {
        let json = r#"{"node_id": "550e8400-e29b-41d4-a716-446655440000", "title": "Test", "content_type": "note", "snippet": "Hello", "score": 0.95}"#;
        let result: SearchResult = serde_json::from_str(json).unwrap();
        assert_eq!(result.title, "Test");
        assert_eq!(result.content_type, "note");
        assert!((result.score - 0.95).abs() < 0.001);
    }

    #[test]
    fn test_search_results_structure() {
        let results = SearchResults {
            vector_results: vec![],
            fts_results: vec![],
            combined: vec![],
        };
        let json = serde_json::to_string(&results).unwrap();
        let parsed: SearchResults = serde_json::from_str(&json).unwrap();
        assert!(parsed.vector_results.is_empty());
        assert!(parsed.fts_results.is_empty());
        assert!(parsed.combined.is_empty());
    }

    #[test]
    fn test_create_node_request_serialization() {
        let req = CreateNodeRequest {
            vault_id: uuid::Uuid::new_v4(),
            title: "Hello".to_string(),
            content: "World".to_string(),
            content_type: Some("note".to_string()),
            file_path: None,
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("Hello"));
        assert!(json.contains("note"));
        let parsed: CreateNodeRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.title, "Hello");
    }

    #[test]
    fn test_update_node_request_partial() {
        let json = r#"{"id": "550e8400-e29b-41d4-a716-446655440000", "content": "New content"}"#;
        let req: UpdateNodeRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.content, Some("New content".to_string()));
        assert!(req.title.is_none());
        assert!(req.content_type.is_none());
    }

    #[test]
    fn test_graph_data_serialization_roundtrip() {
        let data = GraphData {
            nodes: vec![GraphNode {
                id: "n1".to_string(),
                title: "Node 1".to_string(),
                content_type: "note".to_string(),
            }],
            edges: vec![GraphEdge {
                source: "n1".to_string(),
                target: "n2".to_string(),
                label: "LINKS_TO".to_string(),
            }],
        };
        let json = serde_json::to_string(&data).unwrap();
        let parsed: GraphData = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.nodes.len(), 1);
        assert_eq!(parsed.edges.len(), 1);
        assert_eq!(parsed.edges[0].label, "LINKS_TO");
    }

    #[test]
    fn test_chat_message_serialization() {
        let msg = ChatMessage {
            role: "user".to_string(),
            content: "Hello".to_string(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("user"));
        assert!(json.contains("Hello"));
    }

    #[test]
    fn test_chat_response_serialization() {
        let resp = ChatResponse {
            answer: "The answer".to_string(),
            citations: vec![Citation {
                node_id: uuid::Uuid::new_v4(),
                title: "Source".to_string(),
                snippet: "A snippet".to_string(),
            }],
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("The answer"));
        assert!(json.contains("Source"));
    }

    #[test]
    fn test_tag_suggestion_serialization() {
        let tag = TagSuggestion {
            name: "rust".to_string(),
            confidence: 0.95,
            reason: "Topic is about Rust".to_string(),
        };
        let json = serde_json::to_string(&tag).unwrap();
        let parsed: TagSuggestion = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, "rust");
        assert!((parsed.confidence - 0.95).abs() < 0.001);
    }

    #[test]
    fn test_obsidian_import_result_serialization() {
        let result = ObsidianImportResult {
            nodes_created: 10,
            links_created: 5,
            files_scanned: 20,
        };
        let json = serde_json::to_string(&result).unwrap();
        let parsed: ObsidianImportResult = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.nodes_created, 10);
        assert_eq!(parsed.links_created, 5);
        assert_eq!(parsed.files_scanned, 20);
    }

    #[test]
    fn test_extraction_node_serialization() {
        let node = ExtractionNode {
            name: "Entity".to_string(),
            description: "A test entity".to_string(),
            extraction_type: "entity".to_string(),
            confidence: 0.8,
            source_node_id: uuid::Uuid::new_v4(),
            chunk_index: 0,
        };
        let json = serde_json::to_string(&node).unwrap();
        let parsed: ExtractionNode = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, "Entity");
        assert!((parsed.confidence - 0.8).abs() < 0.001);
        assert_eq!(parsed.chunk_index, 0);
    }

    #[test]
    fn test_search_query_optional_fields() {
        let json = r#"{"vault_id": "550e8400-e29b-41d4-a716-446655440000", "query": "test"}"#;
        let query: SearchQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.query, "test");
        assert_eq!(query.limit, None);
        assert_eq!(query.include_fts, None);
    }

    #[test]
    fn test_search_query_with_all_fields() {
        let json = r#"{"vault_id": "550e8400-e29b-41d4-a716-446655440000", "query": "test", "limit": 5, "include_fts": true}"#;
        let query: SearchQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.limit, Some(5));
        assert_eq!(query.include_fts, Some(true));
    }

    #[test]
    fn test_search_result_clone() {
        let sr = SearchResult {
            node_id: uuid::Uuid::new_v4(),
            title: "Test".to_string(),
            content_type: "note".to_string(),
            snippet: "Snippet".to_string(),
            score: 0.5,
        };
        let cloned = sr.clone();
        assert_eq!(cloned.title, sr.title);
        assert_eq!(cloned.score, sr.score);
    }
}
