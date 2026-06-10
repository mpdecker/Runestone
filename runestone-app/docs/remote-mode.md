# Remote Mode

Runestone supports two connection modes:

- **Local** — Desktop app connects directly to PostgreSQL + Neo4j (Docker Compose or native install).
- **Remote** — Client calls the `runestone-server` Axum binary over HTTP. Mobile builds default to remote when local databases are unavailable.

## Architecture

```
┌─────────────┐     POST /api/invoke/{command}     ┌──────────────────┐
│ Tauri client│ ─────────────────────────────────►│ runestone-server │
│ (dispatch)  │     Bearer RUNESTONE_API_TOKEN    │  dispatch_local  │
└─────────────┘                                    └────────┬─────────┘
                                                            │
                                                   PostgreSQL + Neo4j
```

All non-desktop commands route through `src-tauri/src/router.rs`:

1. Desktop-only commands → local handler (errors in remote mode)
2. Local mode with DB pools → `runestone_core::dispatch_local`
3. Remote configured → `remote_api::remote_invoke`

## Server endpoints

| Endpoint | Method | Auth | Description |
|----------|--------|------|-------------|
| `/api/health` | GET | Optional Bearer | Health check |
| `/api/invoke/{command}` | POST | Bearer if `RUNESTONE_API_TOKEN` set | Execute command with JSON body |

## Capability matrix

| Command | Local | Remote | Notes |
|---------|:-----:|:------:|-------|
| `init_database` | ✓ | ✓ | |
| `create_vault` | ✓ | ✓ | |
| `list_vaults` | ✓ | ✓ | |
| `create_node` | ✓ | ✓ | |
| `update_node` | ✓ | ✓ | |
| `delete_node` | ✓ | ✓ | |
| `get_node` | ✓ | ✓ | |
| `list_nodes` | ✓ | ✓ | |
| `get_random_node` | ✓ | ✓ | |
| `get_graph_data` | ✓ | ✓ | |
| `get_local_graph` | ✓ | ✓ | |
| `parse_wiki_links` | ✓ | ✓ | |
| `get_backlinks` | ✓ | ✓ | |
| `get_outgoing_links` | ✓ | ✓ | |
| `semantic_search` | ✓ | ✓ | Requires embedding provider |
| `find_similar` | ✓ | ✓ | |
| `hybrid_search` | ✓ | ✓ | |
| `boolean_search` | ✓ | ✓ | |
| `regex_search` | ✓ | ✓ | |
| `get_node_by_alias` | ✓ | ✓ | |
| `add_alias` / `remove_alias` | ✓ | ✓ | |
| `extract_from_document` | ✓ | ✓ | Requires LLM provider |
| `get_pending_extractions` | ✓ | ✓ | |
| `approve_extraction` | ✓ | ✓ | |
| `reject_extraction` | ✓ | ✓ | |
| `batch_approve_extractions` | ✓ | ✓ | |
| `summarize_node` | ✓ | ✓ | Requires LLM provider |
| `chat_with_graph` | ✓ | ✓ | Requires LLM + embeddings |
| `suggest_tags` | ✓ | ✓ | Requires LLM provider |
| `get_node_tags` | ✓ | ✓ | |
| `add_tags_to_node` | ✓ | ✓ | |
| `remove_tag_from_node` | ✓ | ✓ | |
| `list_tags` | ✓ | ✓ | |
| `get_nodes_by_tag` | ✓ | ✓ | |
| `accept_tag_suggestions` | ✓ | ✓ | |
| `get_node_properties` | ✓ | ✓ | |
| `set_node_property` | ✓ | ✓ | |
| `remove_node_property` | ✓ | ✓ | |
| `create_daily_note` | ✓ | ✓ | |
| `list_templates` | ✓ | ✓ | |
| `create_node_from_template` | ✓ | ✓ | |
| `get_node_versions` | ✓ | ✓ | |
| `restore_node_version` | ✓ | ✓ | |
| `merge_nodes` | ✓ | ✓ | |
| `split_node` | ✓ | ✓ | |
| `scan_vault` | ✓ | ✗ | Desktop-only (filesystem) |
| `import_document` | ✓ | ✗ | Desktop-only (local files) |
| `import_obsidian_vault` | ✓ | ✗ | Desktop-only (filesystem) |
| `export_node_to_markdown` | ✓ | ✗ | Desktop-only (filesystem) |
| `export_vault_to_markdown` | ✓ | ✗ | Desktop-only (filesystem) |
| `start_clipper_server` | ✓ | ✗ | Desktop-only (local HTTP) |
| `stop_clipper_server` | ✓ | ✗ | Desktop-only |
| `get_clipper_status` | ✓ | ✗ | Desktop-only |
| `get_clipper_auth_token` | ✓ | ✗ | Desktop-only |
| `list_available_plugins` | ✓ | ✗ | Desktop-only (filesystem) |
| `read_plugin_file` | ✓ | ✗ | Desktop-only (filesystem) |

## Connection lifecycle

1. **Local startup** — Desktop connects to `DATABASE_URL` / `NEO4J_*`. Mode = `Local`, `connected: true`.
2. **Switch to remote** — `configure_server_connection` clears local pools, sets remote URL/token, `connected: false`.
3. **Test connection** — `test_connection` hits `GET /api/health`; on success sets `connected: true`.
4. **Status** — `get_connection_status` reports mode, URL, and connection flag.

## Docker deployment

```bash
docker compose up -d postgres neo4j runestone-server
```

Set `RUNESTONE_API_TOKEN` in `.env` and configure the desktop/mobile client with:

- Server URL: `http://localhost:3000` (or your host)
- Auth token: same value as `RUNESTONE_API_TOKEN`

## Server hardening

The Axum server applies these defaults (see `runestone-server/src/lib.rs`):

| Setting | Value | Notes |
|---------|-------|-------|
| CORS | All origins, methods, headers | Suitable for dev / trusted networks; restrict at reverse proxy in production |
| Max invoke body | 2 MB | `DefaultBodyLimit` on `/api/invoke/{command}` |
| Port | `3000` | Override with `RUNESTONE_SERVER_PORT` |

Environment variables:

| Variable | Default | Description |
|----------|---------|-------------|
| `DATABASE_URL` | `postgres://runestone:runestone@postgres:5432/runestone` | PostgreSQL connection |
| `NEO4J_URL` | `bolt://neo4j:7687` | Neo4j Bolt URL |
| `NEO4J_USER` / `NEO4J_PASSWORD` | `neo4j` / `runestone` | Neo4j credentials |
| `RUNESTONE_API_TOKEN` | (unset) | When set, all invoke requests require `Authorization: Bearer <token>` |
| `RUNESTONE_SERVER_PORT` | `3000` | HTTP listen port |
| `SEARCH_RERANK` | `false` | Enable optional rerank stage for hybrid search (adds latency) |
