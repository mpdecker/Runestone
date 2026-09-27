//! Cross-store integration tests. They need real PostgreSQL (pgvector) and Neo4j and are skipped
//! (with a message) unless `DATABASE_URL` and `NEO4J_URL` are set, e.g. after
//! `docker compose up -d postgres neo4j`:
//!
//! ```text
//! DATABASE_URL=postgres://runestone:runestone@localhost:5442/runestone \
//! NEO4J_URL=bolt://localhost:7688 cargo test -p runestone-core --test db_integration
//! ```
use runestone_core::dispatch::dispatch_local;
use runestone_core::{BackendContext, EmbeddingConfig, LlmConfig};
use serde_json::{json, Value};

async fn ctx() -> Option<BackendContext> {
    let (Ok(pg_url), Ok(neo_url)) = (std::env::var("DATABASE_URL"), std::env::var("NEO4J_URL"))
    else {
        eprintln!("skipping: DATABASE_URL / NEO4J_URL not set");
        return None;
    };
    let pg = runestone_core::db::create_pg_pool(&pg_url).await.unwrap();
    runestone_core::db::run_pg_migrations(&pg).await.unwrap();
    let user = std::env::var("NEO4J_USER").unwrap_or_else(|_| "neo4j".into());
    let pass = std::env::var("NEO4J_PASSWORD").unwrap_or_else(|_| "runestone".into());
    let neo4j = runestone_core::db::create_neo4j_graph(&neo_url, &user, &pass)
        .await
        .unwrap();
    // Must be repeatable on a database that was already initialised.
    runestone_core::db::run_neo4j_init(&neo4j).await.unwrap();
    runestone_core::db::run_neo4j_init(&neo4j).await.unwrap();
    Some(BackendContext {
        pg,
        neo4j,
        embed_config: EmbeddingConfig::default(),
        llm_config: LlmConfig::default(),
    })
}

async fn call(c: &BackendContext, cmd: &str, args: Value) -> Value {
    dispatch_local(c, cmd, args)
        .await
        .unwrap_or_else(|e| panic!("{cmd} failed: {e}"))
}

async fn vault(c: &BackendContext) -> String {
    let name = format!("it-{}", uuid::Uuid::new_v4());
    call(c, "create_vault", json!({"name": name, "root_path": "/tmp"})).await["id"]
        .as_str()
        .unwrap()
        .to_string()
}

async fn node(c: &BackendContext, v: &str, title: &str, content: &str) -> String {
    call(
        c,
        "create_node",
        json!({"vault_id": v, "title": title, "content": content}),
    )
    .await["id"]
        .as_str()
        .unwrap()
        .to_string()
}

async fn graph(c: &BackendContext, v: &str) -> Value {
    call(c, "get_graph_data", json!({"vault_id": v})).await
}

fn titles(g: &Value) -> Vec<String> {
    let mut t: Vec<String> = g["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n["title"].as_str().unwrap().to_string())
        .collect();
    t.sort();
    t
}

#[tokio::test]
async fn deleting_a_linked_note_keeps_postgres_and_neo4j_in_sync() {
    let Some(c) = ctx().await else { return };
    let v = vault(&c).await;
    let a = node(&c, &v, "A", "<p>see [[B]]</p>").await;
    let b = node(&c, &v, "B", "<p>target</p>").await;
    assert_eq!(graph(&c, &v).await["edges"].as_array().unwrap().len(), 1);

    call(&c, "delete_node", json!(b)).await;

    let g = graph(&c, &v).await;
    assert_eq!(titles(&g), vec!["A"]);
    assert!(g["edges"].as_array().unwrap().is_empty());
    let listed = call(&c, "list_nodes", json!(v)).await;
    assert_eq!(listed.as_array().unwrap().len(), 1);
    // the link is unresolved again, not lost
    let out = call(&c, "get_outgoing_links", json!(a)).await;
    assert_eq!(out[0]["content_type"], "unresolved");
}

#[tokio::test]
async fn wiki_links_follow_the_text() {
    let Some(c) = ctx().await else { return };
    let v = vault(&c).await;
    let a = node(
        &c,
        &v,
        "Src",
        "<p>[[Later]] [[A &amp; B]] [[Tgt|alias]] [[C# notes]]</p>",
    )
    .await;
    node(&c, &v, "Tgt", "x").await;
    node(&c, &v, "A & B", "x").await;
    node(&c, &v, "C# notes", "x").await;
    // "Later" did not exist yet
    assert_eq!(graph(&c, &v).await["edges"].as_array().unwrap().len(), 3);
    node(&c, &v, "Later", "x").await;
    assert_eq!(graph(&c, &v).await["edges"].as_array().unwrap().len(), 4);

    // removing links from the text removes the rows and the edges
    call(&c, "update_node", json!({"id": a, "content": "<p>[[Tgt]]</p>"})).await;
    assert_eq!(graph(&c, &v).await["edges"].as_array().unwrap().len(), 1);
    let out = call(&c, "get_outgoing_links", json!(a)).await;
    assert_eq!(out.as_array().unwrap().len(), 1);
    // re-parsing is idempotent
    call(&c, "parse_wiki_links", json!(a)).await;
    assert_eq!(graph(&c, &v).await["edges"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn versions_are_unique_and_restore_is_undoable() {
    let Some(c) = ctx().await else { return };
    let v = vault(&c).await;
    let n = node(&c, &v, "V", "v0").await;
    let mut tasks = Vec::new();
    for i in 0..15 {
        let c2 = c.clone();
        let n2 = n.clone();
        tasks.push(tokio::spawn(async move {
            dispatch_local(&c2, "update_node", json!({"id": n2, "content": format!("p{i}")}))
                .await
                .unwrap();
        }));
    }
    for t in tasks {
        t.await.unwrap();
    }
    let versions = call(&c, "get_node_versions", json!(n)).await;
    let mut nums: Vec<i64> = versions
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x["version_number"].as_i64().unwrap())
        .collect();
    let len = nums.len();
    nums.sort();
    nums.dedup();
    assert_eq!(nums.len(), len, "duplicate version numbers");

    call(&c, "update_node", json!({"id": n, "content": "latest"})).await;
    let versions = call(&c, "get_node_versions", json!(n)).await;
    let oldest = versions.as_array().unwrap().last().unwrap()["id"].clone();
    call(&c, "restore_node_version", oldest).await;
    let versions = call(&c, "get_node_versions", json!(n)).await;
    assert!(
        versions
            .as_array()
            .unwrap()
            .iter()
            .any(|x| x["content"] == "latest"),
        "restoring must snapshot the content it replaces"
    );
}

#[tokio::test]
async fn merge_and_split_are_safe() {
    let Some(c) = ctx().await else { return };
    let v = vault(&c).await;
    let a = node(&c, &v, "A", "<p>日本語のテキスト</p><p>second 🙂 paragraph</p>").await;
    let b = node(&c, &v, "B", "<p>link [[A]]</p>").await;
    let cc = node(&c, &v, "C", "<p>other</p>").await;

    let err = dispatch_local(&c, "merge_nodes", json!({"source_id": a, "target_id": a}))
        .await
        .unwrap_err();
    assert!(err.contains("itself"), "{err}");
    assert!(dispatch_local(&c, "get_node", json!(a)).await.is_ok());

    // split on multibyte content does not panic and keeps all text
    let res = call(&c, "split_node", json!({"node_id": a, "new_title": "A2"})).await;
    let joined = format!(
        "{}{}",
        res[0]["content"].as_str().unwrap(),
        res[1]["content"].as_str().unwrap()
    );
    assert_eq!(joined, "<p>日本語のテキスト</p><p>second 🙂 paragraph</p>");

    // merge A into C: B's link to A now points at C
    call(&c, "merge_nodes", json!({"source_id": a, "target_id": cc})).await;
    let backlinks = call(&c, "get_backlinks", json!(cc)).await;
    assert!(backlinks
        .as_array()
        .unwrap()
        .iter()
        .any(|x| x["node_id"] == json!(b)));
    assert!(!titles(&graph(&c, &v).await).contains(&"A".to_string()));
}

#[tokio::test]
async fn search_commands_work_end_to_end() {
    let Some(c) = ctx().await else { return };
    let v = vault(&c).await;
    node(&c, &v, "Tuning", "<p>Vacuum and <b>index</b> the tables. Datasets.</p>").await;
    node(&c, &v, "Graphs", "<p>Cypher traverses relationships</p>").await;

    let hybrid = call(&c, "hybrid_search", json!({"vault_id": v, "query": "index"})).await;
    let fts = hybrid["fts_results"].as_array().unwrap();
    assert_eq!(fts.len(), 1);
    assert!(!fts[0]["snippet"].as_str().unwrap().contains('<'));

    for (q, expect) in [
        ("index", 1),
        ("index tables", 1),
        ("index OR cypher", 2),
        ("NOT index", 1),
        ("(", 0),
        ("index AND", 1),
        ("'; drop table nodes;--", 0),
    ] {
        let r = call(&c, "boolean_search", json!({"vault_id": v, "query": q})).await;
        assert_eq!(r.as_array().unwrap().len(), expect, "boolean {q:?}");
    }
    let r = call(&c, "regex_search", json!({"vault_id": v, "pattern": "Vac+u+m"})).await;
    assert_eq!(r.as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn reserved_properties_cannot_poison_tags() {
    let Some(c) = ctx().await else { return };
    let v = vault(&c).await;
    let n = node(&c, &v, "T", "x").await;
    call(&c, "add_tags_to_node", json!({"node_id": n, "tags": ["#Foo", "foo"]})).await;
    let err = dispatch_local(
        &c,
        "set_node_property",
        json!({"node_id": n, "key": "tags", "value": "oops"}),
    )
    .await
    .unwrap_err();
    assert!(err.contains("reserved"), "{err}");
    let tags = call(&c, "list_tags", json!(v)).await;
    assert_eq!(tags.as_array().unwrap().len(), 1);
    call(&c, "remove_tag_from_node", json!({"node_id": n, "tag": "#FOO"})).await;
    assert!(call(&c, "get_node_tags", json!(n)).await["tags"]
        .as_array()
        .unwrap()
        .is_empty());
}

#[tokio::test]
async fn cypher_reads_are_allowed_and_writes_rejected() {
    let Some(c) = ctx().await else { return };
    let v = vault(&c).await;
    node(&c, &v, "Dataset of Sets", "x").await;
    let ok = call(
        &c,
        "run_cypher",
        json!(format!(
            "MATCH (n:Node {{vault_id: '{v}'}}) WHERE n.title CONTAINS 'Dataset' RETURN n.title"
        )),
    )
    .await;
    assert_eq!(ok[0]["values"][0][1], "Dataset of Sets");
    assert!(dispatch_local(&c, "run_cypher", json!("MATCH (n) DETACH DELETE n"))
        .await
        .is_err());
}
