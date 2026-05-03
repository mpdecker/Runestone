CREATE EXTENSION IF NOT EXISTS vector;
CREATE EXTENSION IF NOT EXISTS pg_trgm;
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- ============================================================
-- VAULTS
-- ============================================================
CREATE TABLE vaults (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name        TEXT NOT NULL,
    root_path   TEXT NOT NULL,
    created_at  TIMESTAMPTZ DEFAULT NOW(),
    updated_at  TIMESTAMPTZ DEFAULT NOW()
);

-- ============================================================
-- NODES (content + embedding; graph topology lives in Neo4j)
-- ============================================================
CREATE TABLE nodes (
    id            UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    vault_id      UUID NOT NULL REFERENCES vaults(id) ON DELETE CASCADE,
    title         TEXT NOT NULL,
    content       TEXT NOT NULL DEFAULT '',
    content_type  TEXT NOT NULL DEFAULT 'note',
    embedding     vector(1536),
    file_path     TEXT,
    metadata      JSONB DEFAULT '{}',
    word_count    INTEGER DEFAULT 0,
    created_at    TIMESTAMPTZ DEFAULT NOW(),
    updated_at    TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX idx_nodes_vault      ON nodes(vault_id);
CREATE INDEX idx_nodes_type       ON nodes(content_type);
CREATE INDEX idx_nodes_title_trgm ON nodes USING gin (title gin_trgm_ops);
CREATE INDEX idx_nodes_fts        ON nodes USING gin (
    to_tsvector('english', coalesce(title,'') || ' ' || coalesce(content,''))
);

-- ============================================================
-- DOCUMENT CHUNKS (for LLM extraction & semantic search)
-- ============================================================
CREATE TABLE document_chunks (
    id               UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    document_node_id UUID NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
    chunk_index      INTEGER NOT NULL,
    content          TEXT NOT NULL,
    embedding        vector(1536),
    token_count      INTEGER DEFAULT 0,
    metadata         JSONB DEFAULT '{}',
    created_at       TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX idx_chunks_doc ON document_chunks(document_node_id);

-- ============================================================
-- WIKI-LINKS (parsed [[]] references within notes)
-- ============================================================
CREATE TABLE wiki_links (
    id               UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    source_node_id   UUID NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
    target_title     TEXT NOT NULL,
    resolved_node_id UUID REFERENCES nodes(id),
    context          TEXT,
    created_at       TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX idx_wikilinks_source ON wiki_links(source_node_id);
CREATE INDEX idx_wikilinks_unresolved ON wiki_links(resolved_node_id) WHERE resolved_node_id IS NULL;

-- ============================================================
-- NODE VERSIONS (snapshot history for file recovery)
-- ============================================================
CREATE TABLE node_versions (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    node_id         UUID NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
    version_number  INTEGER NOT NULL,
    title           TEXT NOT NULL,
    content         TEXT NOT NULL,
    word_count      INTEGER NOT NULL DEFAULT 0,
    created_at      TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX idx_versions_node ON node_versions(node_id, version_number DESC);
