// Runestone Neo4j Initialization
// Node indexes
CREATE INDEX node_pg_id    IF NOT EXISTS FOR (n:Node) ON (n.pg_id);
CREATE INDEX node_vault    IF NOT EXISTS FOR (n:Node) ON (n.vault_id);
CREATE INDEX node_type     IF NOT EXISTS FOR (n:Node) ON (n.content_type);
CREATE INDEX node_title    IF NOT EXISTS FOR (n:Node) ON (n.title);

// Tag indexes and constraints
CREATE INDEX tag_name IF NOT EXISTS FOR (t:Tag) ON (t.name);
CREATE CONSTRAINT tag_name_unique IF NOT EXISTS FOR (t:Tag) REQUIRE t.name IS UNIQUE;
