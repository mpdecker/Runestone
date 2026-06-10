---
title: Knowledge Graphs
tags: [concept, graph-theory, data-model]
created: 2026-04-15
aliases: [knowledge graph, KG, graph-based knowledge]
---

# Knowledge Graphs

A **knowledge graph** is a structured representation of information where entities (nodes) are connected by relationships (edges). Unlike traditional databases, knowledge graphs excel at capturing context and semantics.

## Key Components

### Nodes (Entities)
Nodes represent real-world objects, concepts, or pieces of information:
- People, places, organizations (entities)
- Abstract ideas and topics (concepts)
- Documents and notes

### Edges (Relationships)
Edges define how nodes relate to each other:
- `LINKS_TO` — direct connections between notes
- `RELATES_TO` — semantic relationships
- `HAS_TAG` — tag classification
- `REFERENCES` — citation links
- `EXTRACTED_FROM` — AI-extracted knowledge

## Use Cases

1. **Personal Knowledge Management** — Connecting your ideas
2. **Research** — Mapping paper citations and concepts
3. **AI Chat** — Providing context for [[Ollama]] or OpenAI responses
4. **Discovery** — Finding unexpected connections between topics

## Graph Theory Basics

A graph G = (V, E) where:
- V = set of vertices (nodes)
- E = set of edges (relationships)

Runestone's graph is:
- **Directed** — edges have direction (source → target)
- **Labeled** — edges carry relationship types
- **Attributed** — nodes have properties (tags, metadata)

> [!info] Related
> See [[Graph Theory]] for the mathematical foundations.

## Benefits Over Flat Notes

| Traditional Notes | Knowledge Graph |
|---|---|
| Linear reading | Multi-dimensional browsing |
| Manual linking | Automatic backlinks |
| Folder hierarchy | Semantic connections |
| Keyword search | Vector similarity search |
| Static structure | Dynamic visualization |

## See Also

- [[Semantic Search]] for finding related content
- [[Vector Databases]] for the underlying technology
- [[Graph Theory]] for mathematical foundations
- [[Architecture]] for how Runestone implements this
