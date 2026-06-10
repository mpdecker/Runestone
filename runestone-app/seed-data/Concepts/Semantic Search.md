---
title: Semantic Search
tags: [concept, search, vector-database]
created: 2026-04-20
---

# Semantic Search

**Semantic search** goes beyond keyword matching by understanding the meaning and intent behind a query. Instead of looking for exact word matches, it finds content that is _conceptually similar_.

## How It Works

### 1. Embedding Generation
Text is converted into a high-dimensional vector (embedding) using an AI model:
```
"knowledge graph for personal notes"
    ↓ embedding model (e.g., nomic-embed-text via [[Ollama]])
[0.12, -0.45, 0.78, ..., 0.33]  (1536 dimensions)
```

### 2. Vector Storage
Embeddings are stored in [[PostgreSQL]] using the `pgvector` extension:
```sql
CREATE TABLE nodes (
    id UUID PRIMARY KEY,
    title TEXT,
    embedding vector(1536)
);
```

### 3. Similarity Search
Queries are embedded and compared using cosine similarity:
```sql
SELECT title, 1 - (embedding <=> $query_vector) AS similarity
FROM nodes
ORDER BY embedding <=> $query_vector
LIMIT 20;
```

## Search Types in Runestone

| Type | Description | Use Case |
|---|---|---|
| **Semantic** | Cosine similarity on embeddings | "ideas about graph databases" |
| **Full-text** | PostgreSQL tsvector ranking | Exact term matching |
| **Hybrid** | Combined semantic + FTS | Best of both worlds |
| **Boolean** | AND/OR/NOT operators | "graph AND database NOT sql" |
| **Regex** | Pattern matching | Finding specific formatting |

> [!tip] Choosing Search Type
> Use semantic search for exploratory queries and full-text for finding specific terms. Hybrid search gives the best results for most queries.

## See Also

- [[Vector Databases]] for the technical implementation
- [[Knowledge Graphs]] for the data model that search operates on
- [[Architecture]] for the embedding pipeline
