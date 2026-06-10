# Runestone

Personal Knowledge Graph application built with Tauri 2, React 19, PostgreSQL, and Neo4j. Think Obsidian meets a graph database — rich-text notes linked in a semantic knowledge graph with AI-powered extraction, vector search, and Cytoscape visualization.

## Prerequisites

- **Node.js** 18+ (22+ recommended)
- **pnpm** 9+ (`npm install -g pnpm`)
- **Rust** 1.77+ (`rustup install stable`)
- **Docker** and **Docker Compose** (for local databases)
- **Ollama** (optional, for local LLM embeddings and chat — `ollama pull nomic-embed-text llama3.2`)

## Quick Start

```bash
# 1. Clone and install
git clone <repo-url> runestone
cd runestone/runestone-app
pnpm install

# 2. Start databases (PostgreSQL + pgvector + Neo4j)
docker compose up -d

# 3. Launch the app (desktop)
pnpm tauri dev
```

## Seed Data

A set of 15 interconnected sample notes is included for manual testing. Import them via the app's **Obsidian Import** feature:

1. Launch the app (`pnpm tauri dev`)
2. Create a vault pointing to any directory (e.g., `C:\Users\you\RunestoneVault`)
3. Click the **"O"** button in the sidebar toolbar to open the Obsidian import panel
4. Enter the path: `./seed-data` (relative to this repo, e.g. `runestone-app/seed-data`)
5. Click **Import**

All notes will be imported with `[[wiki links]]`, tags, and properties preserved. See the [Manual Testing Checklist](#manual-testing-checklist) section below for what to test.

**Seed data coverage:**

| Folder | Notes | Features Covered |
|---|---|---|
| `Projects/` | Getting Started, Architecture | Code blocks, callouts, tables, wiki links, tags |
| `Concepts/` | Knowledge Graphs, Semantic Search | Aliases, concept types, math notation, SQL examples |
| `Entities/` | Ollama, Neo4j, PostgreSQL | Entity types, Cypher code blocks, setup instructions |
| `Daily/` | 2026-05-03 | Daily note, task lists, checkboxes |
| `Templates/` | Daily Note Template | Template with `{{date}}` variable |
| `Research/` | Vector Databases, Graph Theory | Comparison tables, algorithms, pseudocode |
| `Notes/` | Ideas, Meeting Notes | Checklists, performance benchmarks, action items |
| `Reflections/` | Learning Journey | Personal narrative, reading list |
| `Unsorted/` | Random Note | Node without file path (tests Unsorted bucket) |

## Environment Configuration

Copy `.env.example` to `.env` for defaults or set these environment variables:

| Variable | Default | Description |
|---|---|---|
| `DATABASE_URL` | `postgres://runestone:runestone@localhost:5442/runestone` | PostgreSQL connection string |
| `NEO4J_URL` | `bolt://localhost:7688` | Neo4j Bolt URL |
| `NEO4J_USER` | `neo4j` | Neo4j username |
| `NEO4J_PASSWORD` | `runestone` | Neo4j password |
| `EMBEDDING_PROVIDER` | `ollama` | `ollama` or `openai` |
| `EMBEDDING_MODEL` | `nomic-embed-text` | Model name for embeddings |
| `OLLAMA_BASE_URL` | `http://localhost:11434` | Ollama API base URL |
| `OPENAI_API_KEY` | (empty) | OpenAI API key |
| `OPENAI_BASE_URL` | `https://api.openai.com/v1` | OpenAI-compatible API base URL |
| `LLM_PROVIDER` | `ollama` | `ollama` or `openai` |
| `LLM_MODEL` | `llama3.2` | Model name for chat/summarization |

## Project Structure

```
runestone-app/
├── src/                      # Frontend (React + TypeScript)
│   ├── features/             # Feature components
│   │   ├── chat/             # ChatPanel with RAG
│   │   ├── command-palette/  # Cmd/Ctrl+P command palette
│   │   ├── editor/           # TipTap editor with extensions
│   │   ├── extraction/       # AI extraction review panel
│   │   ├── graph/            # Cytoscape graph canvas
│   │   ├── layout/           # DesktopApp, MobileApp, connection screen
│   │   ├── search/           # Semantic search panel
│   │   ├── sidebar/          # Full sidebar with 12 sub-panels
│   │   └── vault/            # Vault list and creation
│   ├── lib/                  # Utilities (api, types, platform, plugin-manager)
│   ├── store/                # Zustand store (12 slices)
│   └── __tests__/            # Test files
├── src-tauri/                # Backend (Rust + Tauri)
│   ├── src/
│   │   ├── commands/         # 17 command modules (70+ commands)
│   │   ├── models/           # Data models with sqlx::FromRow
│   │   ├── db.rs             # Database pool and migrations
│   │   ├── embedding.rs      # Ollama/OpenAI embedding generation
│   │   ├── llm.rs            # LLM chat and knowledge extraction
│   │   ├── state.rs          # AppState with ConnectionMode
│   │   └── remote_api.rs     # HTTP bridge for remote mode
│   ├── capabilities/         # Tauri permission files
│   └── icons/                # App icons
├── docker-compose.yml        # PostgreSQL + Neo4j services
├── package.json              # Dependencies and scripts
└── docs/                     # Additional documentation
```

## Scripts

| Script | Description |
|---|---|
| `pnpm dev` | Start Vite dev server (frontend only, no Tauri) |
| `pnpm build` | TypeScript check + Vite production build |
| `pnpm tauri dev` | Full desktop app in dev mode |
| `pnpm tauri build` | Production desktop build |
| `pnpm tauri:ios` | iOS dev build (macOS + Xcode required) |
| `pnpm tauri:android` | Android dev build (Android SDK required) |
| `pnpm test` | Run all tests (267 TS + 51 Rust) |
| `pnpm test:watch` | Run tests in watch mode |
| `pnpm lint` | ESLint check |

### Mobile Builds

```bash
# Initialize (one-time)
pnpm tauri ios init     # macOS only, requires Xcode 15+
pnpm tauri android init  # Requires Android SDK + NDK

# Development
pnpm tauri:ios           # iOS Simulator
pnpm tauri:android       # Android Emulator

# Production
pnpm tauri:ios:build
pnpm tauri:android:build
```

See `docs/mobile-build.md` for detailed prerequisites and CI/CD guidance.

## Manual Testing Checklist

### Core Workflow

- [ ] **Create a vault**: Click the vault dropdown → "New Vault" → enter name and path
- [ ] **Create a note**: Click `+` or press `Ctrl+N` → enter title
- [ ] **Edit a note**: Type content in the TipTap editor (supports Markdown shortcuts)
- [ ] **Save a note**: Press `Ctrl+S` or wait for auto-save (2s debounce)
- [ ] **Delete a note**: Press `Ctrl+Shift+D` → confirm dialog
- [ ] **Scan vault**: Click "S" button to import `.md` files from the filesystem

### Rich Text Editor

- [ ] **Markdown shortcuts**: `# ` for heading, `- ` for list, `1. ` for ordered list, `> ` for blockquote, `` ` `` for code
- [ ] **Wiki Links**: Type `[[` to trigger autocomplete → select a note → hover over the link for preview
- [ ] **Slash commands**: Type `/` → choose from h1/h2/h3, bullet, ordered, quote, code, divider, task
- [ ] **Task lists**: Create with `/task` → check/uncheck items
- [ ] **Code blocks**: Create with `/code` → syntax highlighting with lowlight
- [ ] **Tables**: Create with `/table` → resizable columns
- [ ] **Math**: Inline math with `$E=mc^2$` (requires KaTeX)
- [ ] **Mermaid diagrams**: Create a mermaid code block → diagram renders automatically
- [ ] **Callout blocks**: Use `> [!note]`, `> [!warning]`, `> [!info]`, `> [!tip]`, `> [!danger]`
- [ ] **Reading mode**: Toggle Ctrl+E to disable editing
- [ ] **Split pane**: Press Ctrl+Click on a note in the sidebar → opens secondary editor

### Graph View

- [ ] **Global graph**: All nodes displayed with Cytoscape force layout
- [ ] **Local graph**: Double-click a node → shows neighbors (depth 1-5 configurable)
- [ ] **Filter by type**: Use the type filter pills (note/concept/entity/document)
- [ ] **Filter by text**: Type in the filter input to narrow visible nodes
- [ ] **Node colors**: Note=blue, Concept=green, Entity=amber, Document=red
- [ ] **Edge colors**: REFERENCES=slate, CONTAINS=indigo, RELATES_TO=purple, EXTRACTED_FROM=pink, LINKS_TO=teal

### Search

- [ ] **Semantic search**: Type a query → results ranked by vector similarity
- [ ] **Full-text search**: Uses PostgreSQL tsvector ranking
- [ ] **Boolean search**: Supports `AND`, `OR`, `NOT` operators
- [ ] **Regex search**: PostgreSQL regex pattern matching
- [ ] **Similar notes**: Click "Find similar" on a result → finds semantically related notes
- [ ] **Aliases**: Notes accessible by alternative names stored in metadata

### AI Features (requires Ollama or OpenAI)

- [ ] **Summarize node**: Select a note → click "Summarize" in NodeActions → 2-3 sentence summary
- [ ] **Chat with graph**: Open chat panel (Ctrl+L) → ask questions → RAG responses with citations
- [ ] **Suggest tags**: Click "Summarize" → suggested tags appear with confidence scores
- [ ] **Document extraction**: Import a document → extract entities, concepts, and relationships
- [ ] **Extraction review**: Review/approve/reject AI-extracted knowledge

### Version History

- [ ] **Auto-save versions**: Each content change creates a version snapshot
- [ ] **View versions**: Open VersionsPanel → see dated version list
- [ ] **Restore version**: Click "Restore" on a version → overwrites current content

### Node Operations

- [ ] **Merge nodes**: Combines two nodes → appends source content to target, deletes source
- [ ] **Split node**: Splits content at midpoint → creates new node
- [ ] **Backlinks**: Shows all notes linking to the current note
- [ ] **Outgoing links**: Shows all notes the current note links to
- [ ] **Parse Wiki Links**: Extracts `[[links]]` from content and resolves them

### Tags and Properties

- [ ] **Add tags**: Type comma-separated tags in the tag input → Enter to apply
- [ ] **Remove tags**: Hover over a tag → click × to remove
- [ ] **Filter by tag**: Click a tag in TagPane → filters node list
- [ ] **Add property**: Key + type (text/number/bool) + value → Add
- [ ] **Edit property**: Click property value → edit inline → Enter to save / Escape to cancel
- [ ] **Delete property**: Hover → click ×

### Sidebar Panels

- [ ] **Outline**: Shows headings extracted from the editor → click to scroll
- [ ] **File Tree**: Hierarchical view from file paths → expand/collapse folders
- [ ] **List/Flat view**: Toggle between tree and paginated list (50 items/page)
- [ ] **Dark mode**: Toggle in vault header → persists to localStorage
- [ ] **Custom CSS**: CssSnippets panel → live-applied styles
- [ ] **Web Clipper**: Start clipper → POST to `http://localhost:9876/clip` → creates notes

### Obsidian Import

- [ ] Import `.md` files from an Obsidian vault → preserves `[[links]]`
- [ ] Backlinks resolve with second pass → Neo4j edges created

### Daily Notes and Templates

- [ ] **Daily note**: Click "Today" → auto-creates `YYYY-MM-DD` note if not exists
- [ ] **Random note**: Click "Random" → opens a random note
- [ ] **Templates**: Create notes with `content_type = 'template'` → `{{date}}`, `{{time}}`, `{{title}}`, `{{tags}}` replacement

### Keyboard Shortcuts

| Shortcut | Action |
|---|---|
| `Ctrl+S` | Save current note |
| `Ctrl+N` | Create new note |
| `Ctrl+K` | Toggle search panel |
| `Ctrl+O` / `Ctrl+P` | Open command palette |
| `Ctrl+L` | Toggle chat panel |
| `Ctrl+E` | Toggle reading mode |
| `Ctrl+W` | Close current tab |
| `Ctrl+Shift+W` | Close all tabs |
| `Ctrl+Tab` | Next tab |
| `Ctrl+Shift+Tab` | Previous tab |
| `Ctrl+Shift+E` | Toggle extractions panel |
| `Ctrl+Shift+B` | Toggle sidebar |
| `Ctrl+Shift+D` | Delete current note (with confirmation) |

### Error Recovery

- [ ] **Trigger an error**: Force a component error → ErrorBoundary displays "Something went wrong" with "Try Again"
- [ ] **DB unavailable**: Start app without databases → connects in Remote mode (shows connection screen on mobile)

### Mobile (requires platform SDKs)

- [ ] **Connection screen**: First launch → shows server URL input → saves to localStorage
- [ ] **Tab navigation**: Bottom tabs (Graph, Notes, Search, Settings)
- [ ] **Note list**: Search and tap notes to open editor
- [ ] **Search view**: Semantic search with results display
- [ ] **Settings**: Server URL management, dark mode toggle, about section

## Architecture Notes

### Data Layer

- **PostgreSQL + pgvector**: Primary store for nodes, vaults, versions, wiki_links, document chunks, embeddings (1536-dim)
- **Neo4j**: Graph store for relationships (LINKS_TO, HAS_TAG, REFERENCES, CONTAINS, EXTRACTED_FROM, RELATES_TO)
- **Dual-write pattern**: Node creation writes to both PG and Neo4j with rollback on Neo4j failure

### Connection Modes

- **Local mode** (desktop): Direct PG + Neo4j connections
- **Remote mode** (mobile): Thin client connects to a Runestone server over HTTP

### Plugin System

- Plugins loaded from directory with `manifest.json`
- API exposes store access, API calls, hook system, sidebar panel registration, and command registration
- Plugin code loaded via `new Function()` — sandboxed but intended for trusted plugins only

## Testing

```bash
# Run all tests
pnpm test              # 267 TypeScript tests (Vitest + React Testing Library)
cargo test --manifest-path src-tauri/Cargo.toml  # 51 Rust tests

# Watch mode
pnpm test:watch

# Lint
pnpm lint
cargo clippy --manifest-path src-tauri/Cargo.toml
```
