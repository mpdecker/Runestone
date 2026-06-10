CREATE TABLE IF NOT EXISTS embedding_jobs (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    node_id     UUID NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
    status      TEXT NOT NULL DEFAULT 'pending',
    attempts    INTEGER NOT NULL DEFAULT 0,
    last_error  TEXT,
    created_at  TIMESTAMPTZ DEFAULT NOW(),
    updated_at  TIMESTAMPTZ DEFAULT NOW()
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_embedding_jobs_node_unique ON embedding_jobs(node_id);
CREATE INDEX IF NOT EXISTS idx_embedding_jobs_status ON embedding_jobs(status);
