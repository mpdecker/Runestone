---
title: Architecture
tags: [architecture, system-design, backend]
created: 2026-05-02
status: draft
---

# Runestone Architecture

## Overview

Runestone uses a **dual-database** architecture:

1. **[[PostgreSQL]]** with pgvector for primary storage and vector search
2. **[[Neo4j]]** for the graph layer and relationship queries

### Why Two Databases?

PostgreSQL handles:
- CRUD operations on notes
- Full-text search via `tsvector`
- Vector similarity search (pgvector, 1536-dim embeddings)
- Version history
- Document chunking storage

Neo4j handles:
- Graph visualization
- Relationship traversal
- Backlink resolution
- Tag-to-node relationships
- Local graph queries (depth 1-5)

## Data Flow

```
User edits note in TipTap editor
    ↓
Auto-save triggers update_node command
    ↓
Dual write: PostgreSQL (content) + Neo4j (graph)
    ↓
Background task: generate embedding via [[Ollama]]
    ↓
Embedding stored in pgvector column
```

## Embedding Pipeline

```typescript
async function generateEmbedding(text: string) {
  const response = await ollama.embeddings({
    model: 'nomic-embed-text',
    prompt: text,
  })
  return response.embedding
}
```

The embedding dimension is fixed at **1536** (OpenAI ada-002 compatible), though [[Ollama]] models may have different dimensions.

> [!warning] Embedding Mismatch
> If you change the embedding provider, you may need to regenerate all embeddings to match the new dimension.

## See Also

- [[Getting Started with Runestone]] for a user-friendly introduction
- [[Knowledge Graphs]] for graph theory background
- [[Semantic Search]] for how vector search works
- [[Vector Databases]] for database internals
