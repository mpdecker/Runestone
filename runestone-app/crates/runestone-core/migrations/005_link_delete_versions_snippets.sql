-- 1. Deleting a note that other notes link to used to fail with a foreign-key violation
--    (wiki_links.resolved_node_id had no ON DELETE action). Because delete_node removes the
--    Neo4j node first, that failure left PostgreSQL and Neo4j out of sync. A link whose target
--    disappears is simply an unresolved link again.
ALTER TABLE wiki_links DROP CONSTRAINT IF EXISTS wiki_links_resolved_node_id_fkey;
ALTER TABLE wiki_links
    ADD CONSTRAINT wiki_links_resolved_node_id_fkey
    FOREIGN KEY (resolved_node_id) REFERENCES nodes(id) ON DELETE SET NULL;

-- 2. version numbers were computed as MAX()+1 with no constraint, so concurrent saves produced
--    duplicate numbers. Renumber any existing duplicates, then enforce uniqueness.
WITH ranked AS (
    SELECT id, ROW_NUMBER() OVER (PARTITION BY node_id ORDER BY version_number, created_at, id) AS rn
    FROM node_versions
)
UPDATE node_versions v
SET version_number = r.rn
FROM ranked r
WHERE v.id = r.id AND v.version_number <> r.rn;

CREATE UNIQUE INDEX IF NOT EXISTS idx_node_versions_node_number
    ON node_versions (node_id, version_number);

-- 3. Notes are stored as editor HTML; search snippets must be plain text.
CREATE OR REPLACE FUNCTION runestone_snippet(body text, max_chars integer)
RETURNS text
LANGUAGE sql IMMUTABLE AS $$
    SELECT left(
        btrim(
            regexp_replace(
                replace(replace(replace(replace(replace(replace(
                    regexp_replace(coalesce(body, ''), '<[^>]*>', ' ', 'g'),
                    '&nbsp;', ' '), '&lt;', '<'), '&gt;', '>'), '&quot;', '"'), '&#39;', ''''), '&amp;', '&'),
                '\s+', ' ', 'g')
        ),
        max_chars
    )
$$;
