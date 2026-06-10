---
title: Getting Started with Runestone
tags: [getting-started, guide, runestone]
created: 2026-05-01
priority: high
---

# Getting Started with Runestone

Welcome to your personal knowledge graph! Runestone combines the simplicity of markdown notes with the power of a graph database.

## Core Concepts

- Every note is a **node** in a knowledge graph
- Use `[[wiki links]]` to connect related ideas — like [[Knowledge Graphs]]
- Your notes are searchable with **semantic search** via [[Semantic Search]]
- The **graph view** shows how everything connects visually

## Quick Tips

1. Press `Ctrl+N` to create a new note
2. Press `Ctrl+K` to search across all notes
3. Press `Ctrl+S` to save the current note
4. Press `Ctrl+P` or `Ctrl+O` to open the command palette
5. Press `Ctrl+L` to chat with your knowledge graph using AI

## Tech Stack

This app is built with:

- **[[PostgreSQL]]** with pgvector for vector search
- **[[Neo4j]]** for the graph database
- **[[Ollama]]** for local AI embeddings and chat
- **React 19 + TipTap** for the rich text editor

> [!tip] Pro Tip
> Use the tree view to organize notes by folder, or the list view for a flat layout. Toggle with the List/Tree buttons above the note list.

> [!info] Learn More
> Check out [[Architecture]] for technical details about how the dual-database system works.

## Next Steps

- Read [[Architecture]] to understand the system design
- Explore [[Knowledge Graphs]] to learn the theory
- Check out [[Vector Databases]] for search optimization
- Try creating a daily note with the "Today" button
