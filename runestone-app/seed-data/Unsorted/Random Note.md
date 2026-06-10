---
title: Random Thought
tags: [random]
---

# Random Thought

What if we could combine [[Knowledge Graphs]] with [[Vector Databases]] in a single query language?

Something like:

```cypher
MATCH (n:Node)-[:RELATES_TO]->(related)
WHERE vector_similarity(n.embedding, query_embedding) > 0.8
RETURN n, related
ORDER BY vector_similarity DESC
```

This would combine the structural power of [[Neo4j]] with the semantic understanding of pgvector via [[PostgreSQL]].

[[Ollama]] could power the embedding generation, and the [[Architecture]] would need to support cross-database query federation.

Maybe this is already what [[Semantic Search]] + local graph exploration achieves, just not in a single query.

> [!note] Idea Status
> Filed under [[Ideas]] for future exploration.
