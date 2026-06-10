---
title: Meeting Notes - Runestone Architecture Review
tags: [meeting, architecture, planning]
created: 2026-05-01
attendees: [dev-team]
---

# Runestone Architecture Review

**Date:** 2026-05-01  
**Attendees:** Dev Team  

## Agenda

1. Review current [[Architecture]]
2. Discuss [[PostgreSQL]] vs [[Neo4j]] trade-offs
3. Plan [[Semantic Search]] improvements
4. Evaluate [[Ollama]] performance
5. Review [[Knowledge Graphs]] UX

## Discussion Points

### Dual Database Strategy
The team agreed that the dual-database approach adds complexity but provides:
- Best-in-class full-text search (PostgreSQL)
- Fast graph traversals (Neo4j)
- Vector similarity search (pgvector)

> [!warning] Sync Issues
> We need to address the dual-write consistency issue. If PostgreSQL succeeds but Neo4j fails, we have data drift. Consider a transaction log or eventual consistency pattern.

### Search Improvements
Current state: [[Semantic Search]] works well but could improve with:
- Hybrid reranking of results
- Faceted search by content type
- Saved search queries
- Search history

### Performance Benchmarks
```
Operation          | Avg Time | Notes
-------------------|----------|------
Create note        | 45ms     | Dual write
Update note        | 38ms     | With version save
Semantic search    | 120ms    | Over 10k notes
Graph data load    | 200ms    | 500 nodes max
Embedding gen      | 350ms    | Via [[Ollama]]
```

## Action Items

- [ ] Investigate [[Vector Databases]] indexing improvements
- [ ] Prototype search reranking algorithm
- [ ] Add transaction log for dual-write consistency
- [ ] Test with larger datasets (10k+ notes)
- [ ] Review [[Graph Theory]] algorithms for layout optimization
- [ ] Implement saved searches

## Next Meeting
Next week — focus on [[Knowledge Graphs]] visualization and the plugin system.
