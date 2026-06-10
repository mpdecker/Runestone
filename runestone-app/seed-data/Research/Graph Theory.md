---
title: Graph Theory
tags: [research, mathematics, computer-science]
created: 2026-04-18
difficulty: intermediate
---

# Graph Theory

**Graph theory** is the mathematical study of graphs — structures made of vertices (nodes) connected by edges. It provides the theoretical foundation for [[Knowledge Graphs]].

## Basic Definitions

### Graph Types
- **Undirected** — Edges have no direction
- **Directed** (digraph) — Edges point from source to target
- **Weighted** — Edges carry numerical values
- **Bipartite** — Nodes split into two disjoint sets

### Key Concepts
- **Degree** — Number of edges connected to a node
- **Path** — Sequence of edges connecting two nodes
- **Cycle** — Path that starts and ends at the same node
- **Connected component** — Subgraph where all nodes are reachable

## Graph Algorithms

### Traversal
```
BFS (Breadth-First Search):
  Queue: [start_node]
  While queue not empty:
    node = queue.dequeue()
    for each neighbor of node:
      if not visited:
        mark visited
        queue.enqueue(neighbor)

DFS (Depth-First Search):
  Stack: [start_node]
  While stack not empty:
    node = stack.pop()
    for each neighbor of node:
      if not visited:
        mark visited
        stack.push(neighbor)
```

### Centrality Measures
- **Degree centrality** — Number of connections
- **Betweenness centrality** — Bridge nodes between communities
- **Eigenvector centrality** — Importance based on neighbor importance

## Applications in Runestone

The graph view uses these concepts:
- **Layout algorithms** (force-directed, cose) for visualization
- **BFS/DFS** for local graph exploration
- **Shortest path** for relationship discovery
- **Community detection** for topic clustering

## See Also

- [[Knowledge Graphs]] for the application layer
- [[Neo4j]] for graph database implementation
- [[Architecture]] for how graphs power the app
