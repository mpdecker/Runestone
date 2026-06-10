use crate::context::BackendContext;
use crate::embedding::generate_embedding;
use crate::llm::LlmConfig;
use crate::models::chat::{ChatRequest, ChatResponse, Citation, TagSuggestion};
use crate::models::node::Node;
use crate::models::search::SearchResult;
use serde_json::Value;
use uuid::Uuid;

fn http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new())
}

pub async fn simple_chat(prompt: &str, config: &LlmConfig) -> Result<String, String> {
    match config.provider.as_str() {
        "ollama" => ollama_simple_chat(prompt, config).await,
        "openai" => openai_simple_chat(prompt, config).await,
        _ => Err(format!("Unknown LLM provider: {}", config.provider)),
    }
}

async fn ollama_simple_chat(prompt: &str, config: &LlmConfig) -> Result<String, String> {
    let client = http_client();
    let url = format!("{}/api/chat", config.ollama_base_url);

    let body = serde_json::json!({
        "model": config.model,
        "messages": [{"role": "user", "content": prompt}],
        "stream": false,
    });

    let response = client
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Ollama request failed: {}", e))?;

    let result: Value = response
        .json()
        .await
        .map_err(|e| format!("Parse response: {}", e))?;

    result["message"]["content"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "No content in response".to_string())
}

async fn openai_simple_chat(prompt: &str, config: &LlmConfig) -> Result<String, String> {
    let client = http_client();
    let url = format!("{}/chat/completions", config.openai_base_url);

    let body = serde_json::json!({
        "model": config.model,
        "messages": [{"role": "user", "content": prompt}],
    });

    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", config.openai_api_key))
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("OpenAI request failed: {}", e))?;

    let result: Value = response
        .json()
        .await
        .map_err(|e| format!("Parse response: {}", e))?;

    result["choices"][0]["message"]["content"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "No content in response".to_string())
}

pub async fn summarize_node(ctx: &BackendContext, node_id: Uuid) -> Result<String, String> {
    let node = sqlx::query_as::<_, Node>(
        "SELECT id, vault_id, title, content, content_type, file_path, metadata, word_count, created_at, updated_at FROM nodes WHERE id = $1",
    )
    .bind(node_id)
    .fetch_one(&ctx.pg)
    .await
    .map_err(|e| format!("Node not found: {}", e))?;

    let mut neighbors = Vec::new();
    let pg_id = node_id.to_string();
    let mut stream = ctx
        .neo4j
        .execute(
            neo4rs::query(
                "MATCH (n:Node {pg_id: $pg_id})-[r]-(m:Node) RETURN m.title, type(r), m.content_type",
            )
            .param("pg_id", pg_id),
        )
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

    simple_chat(&prompt, &ctx.llm_config).await
}

pub async fn chat_with_graph(
    ctx: &BackendContext,
    request: ChatRequest,
) -> Result<ChatResponse, String> {
    let embedding = generate_embedding(&request.question, &ctx.embed_config).await?;
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
    .fetch_all(&ctx.pg)
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
        let mut stream = ctx
            .neo4j
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
            context_parts.push(format!(
                "\nRelated concepts for '{}':\n{}",
                top.title,
                neighbors.join("\n")
            ));
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
        if history_str.is_empty() {
            "None"
        } else {
            &history_str
        },
        request.question,
    );

    let answer = simple_chat(&prompt, &ctx.llm_config).await?;

    Ok(ChatResponse { answer, citations })
}

fn build_chat_prompt(request: &ChatRequest, context: &str) -> String {
    let mut history_str = String::new();
    for msg in &request.history {
        history_str.push_str(&format!("{}: {}\n", msg.role, msg.content));
    }

    format!(
        "You are a knowledgeable assistant helping with a personal knowledge graph. Answer questions using the provided context.\n\nContext:\n{}\n\nConversation:\n{}\nUser: {}\n\nProvide a concise, helpful answer. Reference specific notes when relevant.",
        context,
        if history_str.is_empty() {
            "None"
        } else {
            &history_str
        },
        request.question,
    )
}

async fn gather_chat_context(
    ctx: &BackendContext,
    request: &ChatRequest,
) -> Result<(String, Vec<Citation>), String> {
    let embedding = generate_embedding(&request.question, &ctx.embed_config).await?;
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
    .fetch_all(&ctx.pg)
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
        let mut stream = ctx
            .neo4j
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
            context_parts.push(format!(
                "\nRelated concepts for '{}':\n{}",
                top.title,
                neighbors.join("\n")
            ));
        }
    }

    Ok((context_parts.join("\n\n"), citations))
}

pub async fn chat_with_graph_stream<F>(
    ctx: &BackendContext,
    request: ChatRequest,
    mut on_chunk: F,
) -> Result<ChatResponse, String>
where
    F: FnMut(&str),
{
    let (context, citations) = gather_chat_context(ctx, &request).await?;
    let prompt = build_chat_prompt(&request, &context);

    let answer = match ctx.llm_config.provider.as_str() {
        "ollama" => {
            stream_ollama_chat(&prompt, &ctx.llm_config, &mut on_chunk).await?
        }
        _ => {
            let answer = simple_chat(&prompt, &ctx.llm_config).await?;
            on_chunk(&answer);
            answer
        }
    };

    Ok(ChatResponse { answer, citations })
}

async fn stream_ollama_chat<F>(
    prompt: &str,
    config: &LlmConfig,
    on_chunk: &mut F,
) -> Result<String, String>
where
    F: FnMut(&str),
{
    let client = http_client();
    let url = format!("{}/api/chat", config.ollama_base_url);
    let body = serde_json::json!({
        "model": config.model,
        "messages": [{"role": "user", "content": prompt}],
        "stream": true,
    });

    let response = client
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Ollama stream failed: {}", e))?;

    let mut full = String::new();
    let text = response.text().await.map_err(|e| format!("Read stream: {}", e))?;

    for line in text.lines() {
        if line.is_empty() {
            continue;
        }
        if let Ok(val) = serde_json::from_str::<Value>(line) {
            if let Some(chunk) = val["message"]["content"].as_str() {
                if !chunk.is_empty() {
                    full.push_str(chunk);
                    on_chunk(chunk);
                }
            }
        }
    }

    Ok(full)
}

pub async fn suggest_tags(
    ctx: &BackendContext,
    node_id: Uuid,
) -> Result<Vec<TagSuggestion>, String> {
    let node = sqlx::query_as::<_, Node>(
        "SELECT id, vault_id, title, content, content_type, file_path, metadata, word_count, created_at, updated_at FROM nodes WHERE id = $1",
    )
    .bind(node_id)
    .fetch_one(&ctx.pg)
    .await
    .map_err(|e| format!("Node not found: {}", e))?;

    let prompt = format!(
        "Suggest 3-5 relevant tags for this note. Return ONLY valid JSON array of objects with fields: name, confidence (0-1), reason.\n\nTitle: {}\nContent: {}",
        node.title,
        &node.content[..1500.min(node.content.len())],
    );

    let response_text = match ctx.llm_config.provider.as_str() {
        "ollama" => {
            let client = reqwest::Client::new();
            let url = format!("{}/api/chat", ctx.llm_config.ollama_base_url);
            let body = serde_json::json!({
                "model": ctx.llm_config.model,
                "messages": [{"role": "user", "content": prompt}],
                "stream": false,
                "format": "json",
            });
            let resp = client
                .post(&url)
                .json(&body)
                .send()
                .await
                .map_err(|e| format!("Ollama: {}", e))?;
            let val: Value = resp.json().await.map_err(|e| format!("Parse: {}", e))?;
            val["message"]["content"]
                .as_str()
                .unwrap_or("[]")
                .to_string()
        }
        "openai" => {
            let client = reqwest::Client::new();
            let url = format!("{}/chat/completions", ctx.llm_config.openai_base_url);
            let body = serde_json::json!({
                "model": ctx.llm_config.model,
                "messages": [{"role": "user", "content": prompt}],
                "response_format": {"type": "json_object"},
            });
            let resp = client
                .post(&url)
                .header(
                    "Authorization",
                    format!("Bearer {}", ctx.llm_config.openai_api_key),
                )
                .json(&body)
                .send()
                .await
                .map_err(|e| format!("OpenAI: {}", e))?;
            let val: Value = resp.json().await.map_err(|e| format!("Parse: {}", e))?;
            val["choices"][0]["message"]["content"]
                .as_str()
                .unwrap_or("[]")
                .to_string()
        }
        _ => return Err("Unknown provider".to_string()),
    };

    let tags: Vec<TagSuggestion> = serde_json::from_str(&response_text).unwrap_or_else(|_| {
        serde_json::from_str(&format!("{{\"tags\": {}}}", response_text))
            .map(|v: Value| {
                v["tags"].as_array().map_or(Vec::new(), |arr| {
                    arr.iter()
                        .filter_map(|t| serde_json::from_value(t.clone()).ok())
                        .collect()
                })
            })
            .unwrap_or_default()
    });

    Ok(tags)
}
