---
title: Vector Databases
tags: [research, database, search]
created: 2026-04-25
status: in-progress
---

# Vector Databases

Vector databases are specialized database systems designed to store, index, and query high-dimensional vector embeddings. They are the foundation of modern [[Semantic Search]].

## Key Concepts

### Embeddings
Every piece of text is converted to a fixed-size vector:
```
"graph database" → [0.12, -0.45, 0.78, 0.01, ..., -0.33]
```

### Distance Metrics
- **Cosine similarity** — Most common for text embeddings
- **Euclidean distance** — Used for visual embeddings
- **Dot product** — For normalized vectors

### Indexing
- **HNSW** (Hierarchical Navigable Small World) — Fast approximate search
- **IVFFlat** — Inverted file with flat compression
- **Exact** — Brute force (accurate but slow)

## pgvector Implementation

Runestone uses [[PostgreSQL]] with pgvector, which supports:

```sql
-- Create a vector column
embedding vector(1536)

-- Cosine distance operator
SELECT * FROM nodes
ORDER BY embedding <=> $query
LIMIT 10;

-- Create an IVFFlat index
CREATE INDEX ON nodes
USING ivfflat (embedding vector_cosine_ops)
WITH (lists = 100);
```

## Comparison

| Database | Type | Best For |
|---|---|---|
| pgvector | PostgreSQL extension | Integrated with existing PG data |
| Pinecone | Managed service | Production, no-ops |
| Weaviate | Open-source | Hybrid search + GraphQL |
| Milvus | Open-source | Billion-scale vectors |
| Qdrant | Open-source | Filtering + payload support |

## See Also

- [[Semantic Search]] for the application layer
- [[PostgreSQL]] for the database implementation
- [[Knowledge Graphs]] for combining vectors with graphs
- [[Ollama]] for generating embeddings locally
