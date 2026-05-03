use crate::embedding::generate_embedding;
use crate::llm::{extract_from_text, LlmConfig};
use crate::models::chat::{ChatRequest, ChatResponse, Citation, TagSuggestion};
use crate::models::node::Node;
use crate::models::search::SearchResult;
use crate::state::AppState;
use uuid::Uuid;

use serde_json::Value;

async fn ollama_simple_chat(prompt: &str, config: &LlmConfig) -> Result<String, String> {
    let client = reqwest::Client::new();
    let url = format!("{}/api/chat", config.ollama_base_url);

    let body = serde_json::json!({
        "model": config.model,
        "messages": [{"role": "user", "content": prompt}],
        "stream": false,
    });

    let response = client.post(&url).json(&body).send().await
        .map_err(|e| format!("Ollama request failed: {}", e))?;

    let result: Value = response.json().await
        .map_err(|e| format!("Parse response: {}", e))?;

    result["message"]["content"].as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "No content in response".to_string())
}

async fn openai_simple_chat(prompt: &str, config: &LlmConfig) -> Result<String, String> {
    let client = reqwest::Client::new();
    let url = format!("{}/chat/completions", config.openai_base_url);

    let body = serde_json::json!({
        "model": config.model,
        "messages": [{"role": "user", "content": prompt}],
    });

    let response = client.post(&url)
        .header("Authorization", format!("Bearer {}", config.openai_api_key))
        .json(&body).send().await
        .map_err(|e| format!("OpenAI request failed: {}", e))?;

    let result: Value = response.json().await
        .map_err(|e| format!("Parse response: {}", e))?;

    result["choices"][0]["message"]["content"].as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "No content in response".to_string())
}

#[tauri::command]
pub async fn summarize_node(
    state: tauri::State<'_, AppState>,
    node_id: Uuid,
) -> Result<String, String> {
    let node = sqlx::query_as::<_, Node>(
        "SELECT id, vault_id, title, content, content_type, file_path, metadata, word_count, created_at, updated_at FROM nodes WHERE id = $1",
    )
    .bind(node_id)
    .fetch_one(state.pg()?)
    .await
    .map_err(|e| format!("Node not found: {}", e))?;

    let mut neighbors = Vec::new();
    let pg_id = node_id.to_string();
    let mut stream = state
        .neo4j()?
        .execute(neo4rs::query("MATCH (n:Node {pg_id: $pg_id})-[r]-(m:Node) RETURN m.title, type(r), m.content_type").param("pg_id", pg_id))
        .await
        .map_err(|e| format!("Neo4j query failed: {}", e))?;

    while let Ok(Some(row)) = stream.next().await {
        let title: String = row.get("m.title").unwrap_or_default();
        let rel_type: String = row.get("type(r)").unwrap_or_default();
        let content_type: String = row.get("m.content_type").unwrap_or_default();
        neighbors.push(format!("- {} ({}): {}", title, content_type, rel_type));
    }

    let neighborhood = if neighbors.is_empty() {
        "No connected nodes.".to_string()
    } else {
        neighbors.join("\n")
    };

    let prompt = format!(
        "Summarize the following note and its connections in 2-3 sentences. Be concise and insightful.\n\nNote title: {}\nContent: {}\n\nConnected nodes:\n{}",
        node.title,
        &node.content[..1000.min(node.content.len())],
        neighborhood,
    );

    let summary = match extract_from_text(&prompt, &state.llm_config).await {
        Ok(_) => {
            match state.llm_config.provider.as_str() {
                "ollama" => ollama_simple_chat(&prompt, &state.llm_config).await?,
                "openai" => openai_simple_chat(&prompt, &state.llm_config).await?,
                _ => return Err(format!("Unknown LLM provider: {}", state.llm_config.provider)),
            }
        }
        Err(e) => return Err(e),
    };

    Ok(summary)
}

#[tauri::command]
pub async fn chat_with_graph(
    state: tauri::State<'_, AppState>,
    request: ChatRequest,
) -> Result<ChatResponse, String> {
    let embedding = generate_embedding(&request.question, &state.embed_config).await?;
    let vector = pgvector::Vector::from(embedding);

    let relevant = sqlx::query_as::<_, SearchResult>(
        r#"SELECT id as node_id, title, content_type,
           substring(content, 1, 300) as snippet,
           1 - (embedding <=> $1) as score
           FROM nodes
           WHERE vault_id = $2 AND embedding IS NOT NULL
           ORDER BY embedding <=> $1
           LIMIT 5"#,
    )
    .bind(&vector)
    .bind(request.vault_id)
    .fetch_all(state.pg()?)
    .await
    .unwrap_or_default();

    let mut context_parts = Vec::new();
    let mut citations = Vec::new();

    for r in &relevant {
        context_parts.push(format!("[{}] {}: {}", r.title, r.content_type, r.snippet));
        citations.push(Citation {
            node_id: r.node_id,
            title: r.title.clone(),
            snippet: r.snippet.clone(),
        });
    }

    if let Some(top) = relevant.first() {
        let mut stream = state
            .neo4j()?
            .execute(
                neo4rs::query("MATCH (n:Node {pg_id: $pg_id})-[r]-(m:Node) RETURN m.title, type(r)")
                    .param("pg_id", top.node_id.to_string()),
            )
            .await
            .map_err(|e| format!("Neo4j failed: {}", e))?;

        let mut neighbors = Vec::new();
        while let Ok(Some(row)) = stream.next().await {
            let title: String = row.get("m.title").unwrap_or_default();
            let rel_type: String = row.get("type(r)").unwrap_or_default();
            neighbors.push(format!("- {} ({})", title, rel_type));
        }
        if !neighbors.is_empty() {
            context_parts.push(format!("\nRelated concepts for '{}':\n{}", top.title, neighbors.join("\n")));
        }
    }

    let context = context_parts.join("\n\n");

    let mut history_str = String::new();
    for msg in &request.history {
        history_str.push_str(&format!("{}: {}\n", msg.role, msg.content));
    }

    let prompt = format!(
        "You are a knowledgeable assistant helping with a personal knowledge graph. Answer questions using the provided context.\n\nContext:\n{}\n\nConversation:\n{}\nUser: {}\n\nProvide a concise, helpful answer. Reference specific notes when relevant.",
        context,
        if history_str.is_empty() { "None" } else { &history_str },
        request.question,
    );

    let answer = match state.llm_config.provider.as_str() {
        "ollama" => ollama_simple_chat(&prompt, &state.llm_config).await?,
        "openai" => openai_simple_chat(&prompt, &state.llm_config).await?,
        _ => return Err("Unknown LLM provider".to_string()),
    };

    Ok(ChatResponse { answer, citations })
}

#[tauri::command]
pub async fn suggest_tags(
    state: tauri::State<'_, AppState>,
    node_id: Uuid,
) -> Result<Vec<TagSuggestion>, String> {
    let node = sqlx::query_as::<_, Node>(
        "SELECT id, vault_id, title, content, content_type, file_path, metadata, word_count, created_at, updated_at FROM nodes WHERE id = $1",
    )
    .bind(node_id)
    .fetch_one(state.pg()?)
    .await
    .map_err(|e| format!("Node not found: {}", e))?;

    let prompt = format!(
        "Suggest 3-5 relevant tags for this note. Return ONLY valid JSON array of objects with fields: name, confidence (0-1), reason.\n\nTitle: {}\nContent: {}",
        node.title,
        &node.content[..1500.min(node.content.len())],
    );

    let response_text = match state.llm_config.provider.as_str() {
        "ollama" => {
            let client = reqwest::Client::new();
            let url = format!("{}/api/chat", state.llm_config.ollama_base_url);
            let body = serde_json::json!({
                "model": state.llm_config.model,
                "messages": [{"role": "user", "content": prompt}],
                "stream": false,
                "format": "json",
            });
            let resp = client.post(&url).json(&body).send().await.map_err(|e| format!("Ollama: {}", e))?;
            let val: Value = resp.json().await.map_err(|e| format!("Parse: {}", e))?;
            val["message"]["content"].as_str().unwrap_or("[]").to_string()
        }
        "openai" => {
            let client = reqwest::Client::new();
            let url = format!("{}/chat/completions", state.llm_config.openai_base_url);
            let body = serde_json::json!({
                "model": state.llm_config.model,
                "messages": [{"role": "user", "content": prompt}],
                "response_format": {"type": "json_object"},
            });
            let resp = client.post(&url).header("Authorization", format!("Bearer {}", state.llm_config.openai_api_key)).json(&body).send().await.map_err(|e| format!("OpenAI: {}", e))?;
            let val: Value = resp.json().await.map_err(|e| format!("Parse: {}", e))?;
            val["choices"][0]["message"]["content"].as_str().unwrap_or("[]").to_string()
        }
        _ => return Err("Unknown provider".to_string()),
    };

    let tags: Vec<TagSuggestion> = serde_json::from_str(&response_text)
        .unwrap_or_else(|_| {
            serde_json::from_str(&format!("{{\"tags\": {}}}", response_text))
                .map(|v: Value| {
                    v["tags"].as_array().map_or(Vec::new(), |arr| {
                        arr.iter().filter_map(|t| serde_json::from_value(t.clone()).ok()).collect()
                    })
                })
                .unwrap_or_default()
        });

    Ok(tags)
}
