---
title: Ollama
tags: [entity, ai, llm, local]
created: 2026-04-01
---

# Ollama

**Ollama** is a tool for running large language models locally on your machine. It provides a simple CLI and REST API for downloading, running, and interacting with open-source models.

## Models Used by Runestone

| Model | Purpose | Size |
|---|---|---|
| `nomic-embed-text` | Text embeddings for semantic search | ~274MB |
| `llama3.2` | Chat, summarization, tag suggestions | ~2GB |

## API Usage

Runestone uses Ollama for:
1. **Embedding generation** — converting notes to vectors for [[Semantic Search]]
2. **Chat with graph** — RAG-style Q&A using [[Knowledge Graphs]] as context
3. **Summarization** — generating 2-3 sentence summaries of notes
4. **Tag suggestions** — AI-powered tag recommendations
5. **Knowledge extraction** — extracting entities, concepts, and relationships from documents

## Setup

```bash
# Install Ollama
brew install ollama    # macOS
# or download from ollama.ai

# Pull models
ollama pull nomic-embed-text
ollama pull llama3.2

# Verify
ollama list
```

> [!note] Configuration
> Set `EMBEDDING_PROVIDER=ollama` and `LLM_PROVIDER=ollama` in your `.env` file to use local models.

## Related

- [[PostgreSQL]] stores the generated embeddings
- [[Neo4j]] stores the extracted relationships
- [[Architecture]] for the embedding pipeline
