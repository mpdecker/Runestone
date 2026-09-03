# Runestone

Personal **knowledge-graph application** — Obsidian-style rich-text notes linked
into a semantic graph, with AI-powered entity extraction, vector search, and
interactive graph visualization.

Built with Tauri 2, React 19, PostgreSQL, and Neo4j; optional local LLM
embeddings and chat via Ollama.

---

## Where the code lives

All application code is in **[`runestone-app/`](runestone-app/)**. That
directory has the full setup, architecture, and development guide:

> **→ [`runestone-app/README.md`](runestone-app/README.md)**

This root directory holds only repository-level documentation and CI.

---

## Status

| | |
| --- | --- |
| Default branch | `main` |
| Remote | `mpdecker/Runsestone` *(note: the remote name contains a typo)* |
| Desktop shell | Tauri 2 (Rust) |
| UI | React 19 + Vite + TypeScript |
| Data | PostgreSQL + Neo4j |
| AI | Ollama (optional, local embeddings + chat) |
| Tests | Vitest |
| CI | `.github/workflows/ci.yml`, `release.yml` |

## Repository map

| Path | Contents |
| --- | --- |
| `runestone-app/src/` | React front end. |
| `runestone-app/src-tauri/` | Tauri/Rust desktop shell. |
| `runestone-app/crates/` | Supporting Rust crates. |
| `runestone-app/extensions/`, `plugins/` | Extension and plugin points. |
| `runestone-app/seed-data/` | Example vault — Concepts, Entities, Daily notes. |
| `runestone-app/docs/` | [`mobile-build.md`](runestone-app/docs/mobile-build.md), [`remote-mode.md`](runestone-app/docs/remote-mode.md). |
| `runestone-app/deploy/`, `tests/` | Deploy config and tests. |

Root-level docs: [`AGENTS.md`](AGENTS.md), [`DEPLOY.md`](DEPLOY.md),
[`READINESS.md`](READINESS.md), and
[`runestone-app/CONTRIBUTING.md`](runestone-app/CONTRIBUTING.md).
