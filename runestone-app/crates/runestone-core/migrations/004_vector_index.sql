CREATE INDEX IF NOT EXISTS idx_nodes_embedding_hnsw
    ON nodes USING hnsw (embedding vector_cosine_ops);
