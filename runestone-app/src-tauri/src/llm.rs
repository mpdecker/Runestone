use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub provider: String,
    pub model: String,
    pub ollama_base_url: String,
    pub openai_api_key: String,
    pub openai_base_url: String,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            provider: std::env::var("LLM_PROVIDER").unwrap_or_else(|_| "ollama".to_string()),
            model: std::env::var("LLM_MODEL").unwrap_or_else(|_| "llama3.2".to_string()),
            ollama_base_url: std::env::var("OLLAMA_BASE_URL").unwrap_or_else(|_| "http://localhost:11434".to_string()),
            openai_api_key: std::env::var("OPENAI_API_KEY").unwrap_or_default(),
            openai_base_url: std::env::var("OPENAI_BASE_URL").unwrap_or_else(|_| "https://api.openai.com/v1".to_string()),
        }
    }
}

#[derive(Debug, Serialize)]
struct OllamaChatRequest {
    model: String,
    messages: Vec<OllamaMessage>,
    stream: bool,
    format: String,
}

#[derive(Debug, Serialize)]
struct OllamaMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct OllamaChatResponse {
    message: OllamaMessageContent,
}

#[derive(Debug, Deserialize)]
struct OllamaMessageContent {
    content: String,
}

#[derive(Debug, Serialize)]
struct OpenAiChatRequest {
    model: String,
    messages: Vec<OpenAiMessage>,
    response_format: OpenAiResponseFormat,
}

#[derive(Debug, Serialize)]
struct OpenAiMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct OpenAiResponseFormat {
    #[serde(rename = "type")]
    format_type: String,
}

#[derive(Debug, Deserialize)]
struct OpenAiChatResponse {
    choices: Vec<OpenAiChoice>,
}

#[derive(Debug, Deserialize)]
struct OpenAiChoice {
    message: OpenAiMessageContent,
}

#[derive(Debug, Deserialize)]
struct OpenAiMessageContent {
    content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionResult {
    pub entities: Vec<ExtractedEntity>,
    pub concepts: Vec<ExtractedConcept>,
    pub relationships: Vec<ExtractedRelationship>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedEntity {
    pub name: String,
    #[serde(rename = "type")]
    pub entity_type: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedConcept {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedRelationship {
    pub source: String,
    pub target: String,
    #[serde(rename = "type")]
    pub rel_type: String,
    pub description: String,
}

const EXTRACTION_PROMPT: &str = r#"You are a knowledge extraction system. Analyze the following text and extract:
1. Entities: named people, organizations, locations, technologies, or specific items mentioned
2. Concepts: abstract ideas, topics, themes, or categories discussed
3. Relationships: connections between entities and concepts

Return ONLY valid JSON in this exact format:
{
  "entities": [
    { "name": "Entity Name", "type": "person|organization|location|technology|other", "description": "Brief description" }
  ],
  "concepts": [
    { "name": "Concept Name", "description": "Brief description" }
  ],
  "relationships": [
    { "source": "Entity or Concept Name", "target": "Entity or Concept Name", "type": "related_to|part_of|depends_on|created_by|uses", "description": "How they are related" }
  ]
}

Text to analyze:
{text}"#;

pub async fn extract_from_text(text: &str, config: &LlmConfig) -> Result<ExtractionResult, String> {
    let prompt = EXTRACTION_PROMPT.replace("{text}", text);

    match config.provider.as_str() {
        "ollama" => ollama_extract(&prompt, config).await,
        "openai" => openai_extract(&prompt, config).await,
        _ => Err(format!("Unknown LLM provider: {}", config.provider)),
    }
}

async fn ollama_extract(prompt: &str, config: &LlmConfig) -> Result<ExtractionResult, String> {
    let client = reqwest::Client::new();
    let url = format!("{}/api/chat", config.ollama_base_url);

    let request = OllamaChatRequest {
        model: config.model.clone(),
        messages: vec![OllamaMessage {
            role: "user".to_string(),
            content: prompt.to_string(),
        }],
        stream: false,
        format: "json".to_string(),
    };

    let response = client
        .post(&url)
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("Ollama request failed: {}", e))?;

    let body = response.text().await.map_err(|e| format!("Read body: {}", e))?;
    let result: OllamaChatResponse = serde_json::from_str(&body)
        .map_err(|e| format!("Parse Ollama response: {} - body: {}", e, &body[..200.min(body.len())]))?;

    let extraction: ExtractionResult = serde_json::from_str(&result.message.content)
        .map_err(|e| format!("Parse extraction JSON: {} - content: {}", e, &result.message.content[..200.min(result.message.content.len())]))?;

    Ok(extraction)
}

async fn openai_extract(prompt: &str, config: &LlmConfig) -> Result<ExtractionResult, String> {
    let client = reqwest::Client::new();
    let url = format!("{}/chat/completions", config.openai_base_url);

    let request = OpenAiChatRequest {
        model: config.model.clone(),
        messages: vec![OpenAiMessage {
            role: "user".to_string(),
            content: prompt.to_string(),
        }],
        response_format: OpenAiResponseFormat {
            format_type: "json_object".to_string(),
        },
    };

    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", config.openai_api_key))
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("OpenAI request failed: {}", e))?;

    let body = response.text().await.map_err(|e| format!("Read body: {}", e))?;
    let result: OpenAiChatResponse = serde_json::from_str(&body)
        .map_err(|e| format!("Parse OpenAI response: {} - body: {}", e, &body[..200.min(body.len())]))?;

    let content = &result.choices.first()
        .ok_or("No choices in OpenAI response")?
        .message.content;

    let extraction: ExtractionResult = serde_json::from_str(content)
        .map_err(|e| format!("Parse extraction JSON: {} - content: {}", e, &content[..200.min(content.len())]))?;

    Ok(extraction)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_llm_config_default_provider() {
        let config = LlmConfig::default();
        assert!(config.provider == "ollama" || config.provider == "openai");
    }

    #[test]
    fn test_llm_config_default_model() {
        let config = LlmConfig::default();
        assert!(!config.model.is_empty());
    }

    #[test]
    fn test_llm_config_default_urls() {
        let config = LlmConfig::default();
        assert!(config.ollama_base_url.starts_with("http"));
        assert!(config.openai_base_url.starts_with("https"));
    }

    #[test]
    fn test_extraction_prompt_contains_placeholder() {
        assert!(EXTRACTION_PROMPT.contains("{text}"));
    }

    #[test]
    fn test_extraction_prompt_not_empty() {
        assert!(!EXTRACTION_PROMPT.is_empty());
        assert!(EXTRACTION_PROMPT.len() > 100);
    }

    #[test]
    fn test_ollama_chat_request_serialization() {
        let req = OllamaChatRequest {
            model: "llama3.2".to_string(),
            messages: vec![OllamaMessage {
                role: "user".to_string(),
                content: "Hello".to_string(),
            }],
            stream: false,
            format: "json".to_string(),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("llama3.2"));
        assert!(json.contains("user"));
        assert!(json.contains("Hello"));
        assert!(json.contains("stream"));
    }

    #[test]
    fn test_openai_chat_request_serialization() {
        let req = OpenAiChatRequest {
            model: "gpt-4o".to_string(),
            messages: vec![OpenAiMessage {
                role: "user".to_string(),
                content: "Hello".to_string(),
            }],
            response_format: OpenAiResponseFormat {
                format_type: "json_object".to_string(),
            },
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("gpt-4o"));
        assert!(json.contains("json_object"));
    }

    #[test]
    fn test_extraction_result_deserialization() {
        let json = r#"{
            "entities": [{"name": "John", "type": "person", "description": "A person"}],
            "concepts": [{"name": "AI", "description": "Artificial Intelligence"}],
            "relationships": [{"source": "John", "target": "AI", "type": "related_to", "description": "works with"}]
        }"#;
        let result: ExtractionResult = serde_json::from_str(json).unwrap();
        assert_eq!(result.entities.len(), 1);
        assert_eq!(result.entities[0].name, "John");
        assert_eq!(result.entities[0].entity_type, "person");
        assert_eq!(result.concepts.len(), 1);
        assert_eq!(result.concepts[0].name, "AI");
        assert_eq!(result.relationships.len(), 1);
        assert_eq!(result.relationships[0].source, "John");
        assert_eq!(result.relationships[0].rel_type, "related_to");
    }

    #[test]
    fn test_extraction_result_empty() {
        let json = r#"{"entities": [], "concepts": [], "relationships": []}"#;
        let result: ExtractionResult = serde_json::from_str(json).unwrap();
        assert!(result.entities.is_empty());
        assert!(result.concepts.is_empty());
        assert!(result.relationships.is_empty());
    }

    #[test]
    fn test_ollama_chat_response_deserialization() {
        let json = r#"{"message": {"content": "Hello back"}}"#;
        let response: OllamaChatResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.message.content, "Hello back");
    }

    #[test]
    fn test_openai_chat_response_deserialization() {
        let json = r#"{"choices": [{"message": {"content": "Hi there"}}]}"#;
        let response: OpenAiChatResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.choices[0].message.content, "Hi there");
    }

    #[test]
    fn test_llm_config_serialization_roundtrip() {
        let config = LlmConfig {
            provider: "ollama".to_string(),
            model: "llama3.2".to_string(),
            ollama_base_url: "http://localhost:11434".to_string(),
            openai_api_key: "sk-test".to_string(),
            openai_base_url: "https://api.openai.com/v1".to_string(),
        };
        let json = serde_json::to_string(&config).unwrap();
        let parsed: LlmConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.provider, "ollama");
        assert_eq!(parsed.model, "llama3.2");
        assert_eq!(parsed.ollama_base_url, "http://localhost:11434");
        assert_eq!(parsed.openai_api_key, "sk-test");
    }
}
