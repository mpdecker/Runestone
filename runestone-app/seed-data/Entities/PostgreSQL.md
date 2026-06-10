---
title: PostgreSQL
tags: [entity, database, relational]
created: 2026-04-03
version: 16
---

# PostgreSQL

**PostgreSQL** is an advanced open-source relational database. In Runestone, it's extended with `pgvector` for vector similarity search and serves as the **primary data store**.

## Extensions Used

```sql
CREATE EXTENSION vector;       -- pgvector for embeddings
CREATE EXTENSION pg_trgm;      -- Trigram indexing for fuzzy text search
CREATE EXTENSION "uuid-ossp";  -- UUID generation
```

## Schema

### Core Tables
- **vaults** — Top-level organization containers
- **nodes** — All notes, concepts, entities, and documents
- **wiki_links** — Parsed `[[link]]` references between nodes
- **node_versions** — Historical snapshots of note changes
- **document_chunks** — Chunked document content for processing

### Key Indexes
```sql
-- Full-text search index
CREATE INDEX idx_nodes_fts ON nodes
  USING gin (to_tsvector('english',
    coalesce(title,'') || ' ' || coalesce(content,'')));

-- Trigram index for fuzzy title matching
CREATE INDEX idx_nodes_title_trgm ON nodes
  USING gin (title gin_trgm_ops);
```

## Search Capabilities

PostgreSQL provides four search types in Runestone:
1. **Semantic** (pgvector `<->` or `<=>` operator)
2. **Full-text** (tsvector/tsquery)
3. **Hybrid** (combined semantic + FTS)
4. **Boolean** (AND/OR/NOT with tsquery)
5. **Regex** (`~` and `~*` operators)

> [!info] Embedding Dimension
> The `embedding` column uses `vector(1536)`, matching OpenAI's `text-embedding-ada-002`. If using [[Ollama]] with a different model, you may need to alter the column or use a different table.

## Related

- [[Neo4j]] for the graph database layer
- [[Vector Databases]] for the theory behind pgvector
- [[Architecture]] for how PostgreSQL fits into the system
