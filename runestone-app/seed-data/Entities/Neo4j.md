---
title: Neo4j
tags: [entity, database, graph]
created: 2026-04-05
version: 5.x
---

# Neo4j

**Neo4j** is a native graph database that stores data as nodes, relationships, and properties. It uses the Cypher query language, which is designed for traversing connected data.

## In Runestone

Neo4j serves as the **graph layer** of the dual-database architecture described in [[Architecture]]. It handles:

### Node Indexes
```cypher
CREATE INDEX node_pg_id IF NOT EXISTS FOR (n:Node) ON (n.pg_id);
CREATE INDEX node_vault IF NOT EXISTS FOR (n:Node) ON (n.vault_id);
CREATE INDEX node_type IF NOT EXISTS FOR (n:Node) ON (n.content_type);
CREATE INDEX node_title IF NOT EXISTS FOR (n:Node) ON (n.title);
```

### Relationship Types
- `LINKS_TO` — wiki link connections between notes
- `HAS_TAG` — node-to-tag classification
- `RELATES_TO` — semantic relationships (AI-extracted)
- `EXTRACTED_FROM` — entity/concept provenance

### Graph Queries
```cypher
// Get local graph (depth 1-5)
MATCH (center:Node {pg_id: $pg_id})-[r*1..$depth]-(neighbor:Node)
RETURN center, neighbor, r

// Get backlinks
MATCH (n:Node)-[:LINKS_TO]->(target:Node {pg_id: $pg_id})
RETURN n.pg_id
```

## Related

- [[PostgreSQL]] for the primary data store
- [[Knowledge Graphs]] for the conceptual model
- [[Graph Theory]] for mathematical foundations
