CREATE UNIQUE INDEX IF NOT EXISTS idx_wiki_links_source_target
    ON wiki_links (source_node_id, target_title);
