use serde::{Deserialize, Serialize};
use std::time::Duration;

const HTTP_TIMEOUT_SECS: u64 = 30;

fn http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(HTTP_TIMEOUT_SECS))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    pub provider: String,
    pub model: String,
    pub ollama_base_url: String,
    pub openai_api_key: String,
    pub openai_base_url: String,
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            provider: std::env::var("EMBEDDING_PROVIDER").unwrap_or_else(|_| "ollama".to_string()),
            model: std::env::var("EMBEDDING_MODEL")
                .unwrap_or_else(|_| "nomic-embed-text".to_string()),
            ollama_base_url: std::env::var("OLLAMA_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:11434".to_string()),
            openai_api_key: std::env::var("OPENAI_API_KEY").unwrap_or_default(),
            openai_base_url: std::env::var("OPENAI_BASE_URL")
                .unwrap_or_else(|_| "https://api.openai.com/v1".to_string()),
        }
    }
}

#[derive(Debug, Serialize)]
struct OllamaEmbedRequest {
    model: String,
    prompt: String,
}

#[derive(Debug, Deserialize)]
struct OllamaEmbedResponse {
    embedding: Vec<f64>,
}

#[derive(Debug, Serialize)]
struct OpenAiEmbedRequest {
    model: String,
    input: String,
    encoding_format: String,
}

#[derive(Debug, Deserialize)]
struct OpenAiEmbedResponse {
    data: Vec<OpenAiEmbedData>,
}

#[derive(Debug, Deserialize)]
struct OpenAiEmbedData {
    embedding: Vec<f32>,
}

/// Width of the `embedding` columns (`vector(1536)` in migrations/001_init.sql).
pub const EMBEDDING_DIMENSIONS: usize = 1536;

/// Make a provider's vector fit the fixed-width pgvector column. The default local model
/// (`nomic-embed-text`) returns 768 numbers; storing that into `vector(1536)` fails, which
/// silently left every note without an embedding. Zero-padding leaves dot products, norms and
/// therefore cosine distance unchanged, so shorter vectors are padded; a model that returns
/// *more* than the column can hold is reported clearly instead of failing inside SQL.
pub fn fit_dimensions(mut embedding: Vec<f32>) -> Result<Vec<f32>, String> {
    if embedding.is_empty() {
        return Err("Embedding provider returned an empty vector".to_string());
    }
    if embedding.len() > EMBEDDING_DIMENSIONS {
        return Err(format!(
            "Embedding model returned {} dimensions but the database column holds at most {}. Pick a model with <= {} dimensions (EMBEDDING_MODEL).",
            embedding.len(),
            EMBEDDING_DIMENSIONS,
            EMBEDDING_DIMENSIONS
        ));
    }
    embedding.resize(EMBEDDING_DIMENSIONS, 0.0);
    Ok(embedding)
}

pub async fn generate_embedding(text: &str, config: &EmbeddingConfig) -> Result<Vec<f32>, String> {
    let raw = match config.provider.as_str() {
        "ollama" => ollama_embed(text, config).await?,
        "openai" => openai_embed(text, config).await?,
        _ => return Err(format!("Unknown embedding provider: {}", config.provider)),
    };
    fit_dimensions(raw)
}

async fn ollama_embed(text: &str, config: &EmbeddingConfig) -> Result<Vec<f32>, String> {
    let client = http_client();
    let url = format!("{}/api/embeddings", config.ollama_base_url);

    let request = OllamaEmbedRequest {
        model: config.model.clone(),
        prompt: text.to_string(),
    };

    let response = client
        .post(&url)
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("Ollama request failed: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("Ollama API error {}: {}", status, body));
    }

    let result: OllamaEmbedResponse = response
        .json()
        .await
        .map_err(|e| format!("Ollama response parsing failed: {}", e))?;

    Ok(result.embedding.into_iter().map(|v| v as f32).collect())
}

async fn openai_embed(text: &str, config: &EmbeddingConfig) -> Result<Vec<f32>, String> {
    let client = http_client();
    let url = format!("{}/embeddings", config.openai_base_url);

    let request = OpenAiEmbedRequest {
        model: config.model.clone(),
        input: text.to_string(),
        encoding_format: "float".to_string(),
    };

    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", config.openai_api_key))
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("OpenAI request failed: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("OpenAI API error {}: {}", status, body));
    }

    let result: OpenAiEmbedResponse = response
        .json()
        .await
        .map_err(|e| format!("OpenAI response parsing failed: {}", e))?;

    if let Some(first) = result.data.into_iter().next() {
        Ok(first.embedding)
    } else {
        Err("OpenAI returned no embedding data".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fit_dimensions_pads_short_vectors_and_preserves_cosine() {
        let a = vec![0.5_f32, -1.0, 2.0];
        let b = vec![1.5_f32, 0.25, -0.5];
        let cos = |x: &[f32], y: &[f32]| {
            let dot: f32 = x.iter().zip(y).map(|(p, q)| p * q).sum();
            let nx: f32 = x.iter().map(|p| p * p).sum::<f32>().sqrt();
            let ny: f32 = y.iter().map(|p| p * p).sum::<f32>().sqrt();
            dot / (nx * ny)
        };
        let pa = fit_dimensions(a.clone()).unwrap();
        let pb = fit_dimensions(b.clone()).unwrap();
        assert_eq!(pa.len(), EMBEDDING_DIMENSIONS);
        assert!((cos(&a, &b) - cos(&pa, &pb)).abs() < 1e-6);
    }

    #[test]
    fn fit_dimensions_rejects_empty_and_oversized() {
        assert!(fit_dimensions(vec![]).is_err());
        let err = fit_dimensions(vec![0.1; EMBEDDING_DIMENSIONS + 1]).unwrap_err();
        assert!(err.contains("1537"), "{err}");
        assert_eq!(
            fit_dimensions(vec![0.1; EMBEDDING_DIMENSIONS])
                .unwrap()
                .len(),
            EMBEDDING_DIMENSIONS
        );
    }

    #[test]
    fn test_embedding_config_default_provider() {
        let config = EmbeddingConfig::default();
        assert!(config.provider == "ollama" || config.provider == "openai");
    }

    #[test]
    fn test_embedding_config_default_model() {
        let config = EmbeddingConfig::default();
        assert!(!config.model.is_empty());
    }

    #[test]
    fn test_embedding_config_default_urls() {
        let config = EmbeddingConfig::default();
        assert!(config.ollama_base_url.starts_with("http"));
        assert!(config.openai_base_url.starts_with("https"));
    }

    #[test]
    fn test_embedding_config_serialization_roundtrip() {
        let config = EmbeddingConfig {
            provider: "openai".to_string(),
            model: "text-embedding-3-small".to_string(),
            ollama_base_url: "http://localhost:11434".to_string(),
            openai_api_key: "sk-test".to_string(),
            openai_base_url: "https://api.openai.com/v1".to_string(),
        };
        let json = serde_json::to_string(&config).unwrap();
        let parsed: EmbeddingConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.provider, "openai");
        assert_eq!(parsed.model, "text-embedding-3-small");
        assert_eq!(parsed.openai_api_key, "sk-test");
    }

    #[test]
    fn test_ollama_embed_request_serialization() {
        let req = OllamaEmbedRequest {
            model: "nomic-embed-text".to_string(),
            prompt: "test text".to_string(),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("nomic-embed-text"));
        assert!(json.contains("prompt"));
    }

    #[test]
    fn test_openai_embed_request_serialization() {
        let req = OpenAiEmbedRequest {
            model: "text-embedding-3-small".to_string(),
            input: "test".to_string(),
            encoding_format: "float".to_string(),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("text-embedding-3-small"));
        assert!(json.contains("float"));
        assert!(json.contains("input"));
    }

    #[test]
    fn test_ollama_embed_response_deserialization() {
        let json = r#"{"embedding": [0.1, 0.2, 0.3]}"#;
        let response: OllamaEmbedResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.embedding.len(), 3);
        assert!((response.embedding[0] - 0.1).abs() < 0.001);
    }

    #[test]
    fn test_openai_embed_response_deserialization() {
        let json = r#"{"data": [{"embedding": [1.0, 2.0]}]}"#;
        let response: OpenAiEmbedResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.data.len(), 1);
        assert_eq!(response.data[0].embedding.len(), 2);
    }
}
