use crate::router::dispatch;
use crate::models::node::Node;
use crate::path_guard::ensure_within_root;
use crate::state::AppState;
use uuid::Uuid;

pub fn html_to_markdown(html: &str) -> String {
    let tags = [
        ("<h1>", "# "),
        ("</h1>", "\n"),
        ("<h2>", "## "),
        ("</h2>", "\n"),
        ("<h3>", "### "),
        ("</h3>", "\n"),
        ("<h4>", "#### "),
        ("</h4>", "\n"),
        ("<h5>", "##### "),
        ("</h5>", "\n"),
        ("<h6>", "###### "),
        ("</h6>", "\n"),
        ("<strong>", "**"),
        ("</strong>", "**"),
        ("<b>", "**"),
        ("</b>", "**"),
        ("<em>", "*"),
        ("</em>", "*"),
        ("<i>", "*"),
        ("</i>", "*"),
        ("<code>", "`"),
        ("</code>", "`"),
        ("<pre>", "```\n"),
        ("</pre>", "\n```\n"),
        ("<blockquote>", "> "),
        ("</blockquote>", "\n"),
        ("<li>", "- "),
        ("</li>", "\n"),
        ("<ol>", ""),
        ("</ol>", "\n"),
        ("<ul>", ""),
        ("</ul>", "\n"),
        ("<p>", ""),
        ("</p>", "\n\n"),
        ("<br>", "\n"),
        ("<br/>", "\n"),
        ("<br />", "\n"),
        ("<hr>", "\n---\n"),
        ("<hr/>", "\n---\n"),
    ];

    let mut s = html.to_string();

    for (tag, replacement) in &tags {
        s = s.replace(tag, replacement);
    }

    let re = regex::Regex::new(r#"<a\s+href="([^"]*)"[^>]*>([^<]*)</a>"#).unwrap();
    s = re.replace_all(&s, "[$2]($1)").to_string();

    let re_img = regex::Regex::new(r#"<img\s+src="([^"]*)"[^>]*alt="([^"]*)"[^>]*/?>"#).unwrap();
    s = re_img.replace_all(&s, "![$2]($1)").to_string();

    let re_strip = regex::Regex::new(r"<[^>]+>").unwrap();
    s = re_strip.replace_all(&s, "").to_string();

    s = s
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'");

    let re_newlines = regex::Regex::new(r"\n{3,}").unwrap();
    s = re_newlines.replace_all(&s, "\n\n").to_string();

    let mut result = s.trim().to_string();
    result.push('\n');

    result
}

#[tauri::command]
pub async fn export_node_to_markdown(
    state: tauri::State<'_, AppState>,
    node_id: Uuid,
    export_path: Option<String>,
) -> Result<String, String> {
    dispatch(
        &state,
        "export_node_to_markdown",
        serde_json::json!({ "node_id": node_id, "export_path": export_path }),
    )
    .await
}

pub async fn export_node_to_markdown_impl(
    state: &AppState,
    node_id: Uuid,
    export_path: Option<String>,
) -> Result<String, String> {
    let node = sqlx::query_as::<_, Node>(
        "SELECT id, vault_id, title, content, content_type, file_path, metadata, word_count, created_at, updated_at FROM nodes WHERE id = $1",
    )
    .bind(node_id)
    .fetch_one(&state.pg()?)
    .await
    .map_err(|e| format!("Node not found: {}", e))?;

    let md_content = html_to_markdown(&node.content);

    let metadata_json = node.metadata.unwrap_or(serde_json::json!({}));
    let mut frontmatter = String::from("---\n");
    frontmatter.push_str(&format!("title: {}\n", node.title));
    if let Some(created) = node.created_at {
        frontmatter.push_str(&format!("created: {}\n", created.format("%Y-%m-%d")));
    }
    if let Some(updated) = node.updated_at {
        frontmatter.push_str(&format!("updated: {}\n", updated.format("%Y-%m-%d")));
    }
    if let Some(tags) = metadata_json.get("tags").and_then(|v| v.as_array()) {
        let tag_list: Vec<&str> = tags.iter().filter_map(|t| t.as_str()).collect();
        if !tag_list.is_empty() {
            frontmatter.push_str(&format!("tags: [{}]\n", tag_list.join(", ")));
        }
    }
    frontmatter.push_str(&format!("content_type: {}\n", node.content_type));
    frontmatter.push_str("---\n\n");

    let full_content = format!("{}{}", frontmatter, md_content);

    let vault_root = sqlx::query_as::<_, (String,)>("SELECT root_path FROM vaults WHERE id = $1")
        .bind(node.vault_id)
        .fetch_one(&state.pg()?)
        .await
        .map_err(|e| format!("Vault not found: {}", e))?
        .0;

    let default_name = format!("{}.md", node.title);
    let path = export_path.unwrap_or(default_name);
    let path = if path.ends_with(".md") {
        path
    } else {
        format!("{}.md", path)
    };

    let full_path = if std::path::Path::new(&path).is_absolute() {
        path
    } else {
        format!(
            "{}/{}",
            vault_root.trim_end_matches('/').trim_end_matches('\\'),
            path
        )
    };

    let safe_path = ensure_within_root(&vault_root, &full_path).map_err(|e| e.to_string())?;

    std::fs::write(&safe_path, &full_content)
        .map_err(|e| format!("Failed to write file: {}", e))?;

    Ok(safe_path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn export_vault_to_markdown(
    state: tauri::State<'_, AppState>,
    vault_id: Uuid,
    output_dir: String,
) -> Result<Vec<String>, String> {
    dispatch(
        &state,
        "export_vault_to_markdown",
        serde_json::json!({ "vault_id": vault_id, "output_dir": output_dir }),
    )
    .await
}

pub async fn export_vault_to_markdown_impl(
    state: &AppState,
    vault_id: Uuid,
    output_dir: String,
) -> Result<Vec<String>, String> {
    let nodes = sqlx::query_as::<_, Node>(
        "SELECT id, vault_id, title, content, content_type, file_path, metadata, word_count, created_at, updated_at FROM nodes WHERE vault_id = $1",
    )
    .bind(vault_id)
    .fetch_all(&state.pg()?)
    .await
    .map_err(|e| format!("Failed to list nodes: {}", e))?;

    let vault_root = sqlx::query_as::<_, (String,)>("SELECT root_path FROM vaults WHERE id = $1")
        .bind(vault_id)
        .fetch_one(&state.pg()?)
        .await
        .map_err(|e| format!("Vault not found: {}", e))?
        .0;

    let safe_output = ensure_within_root(&vault_root, &output_dir).map_err(|e| e.to_string())?;
    let output_dir = safe_output.to_string_lossy().to_string();

    std::fs::create_dir_all(&output_dir)
        .map_err(|e| format!("Failed to create output directory: {}", e))?;

    let mut exported = Vec::new();

    for node in nodes {
        let safe_name = node
            .title
            .replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_");
        let file_path = format!("{}/{}.md", output_dir, safe_name);

        let md_content = html_to_markdown(&node.content);

        let metadata_json = node.metadata.unwrap_or(serde_json::json!({}));
        let mut frontmatter = String::from("---\n");
        frontmatter.push_str(&format!("title: {}\n", node.title));
        if let Some(created) = node.created_at {
            frontmatter.push_str(&format!("created: {}\n", created.format("%Y-%m-%d")));
        }
        if let Some(tags) = metadata_json.get("tags").and_then(|v| v.as_array()) {
            let tag_list: Vec<&str> = tags.iter().filter_map(|t| t.as_str()).collect();
            if !tag_list.is_empty() {
                frontmatter.push_str(&format!("tags: [{}]\n", tag_list.join(", ")));
            }
        }
        frontmatter.push_str(&format!("content_type: {}\n", node.content_type));
        frontmatter.push_str("---\n\n");

        let full_content = format!("{}{}", frontmatter, md_content);

        std::fs::write(&file_path, &full_content)
            .map_err(|e| format!("Failed to write {}: {}", file_path, e))?;

        exported.push(file_path);
    }

    Ok(exported)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn html_to_markdown_converts_headings() {
        let md = html_to_markdown("<h1>Title</h1><p>Body</p>");
        assert!(md.contains("# Title"));
        assert!(md.contains("Body"));
    }

    #[test]
    fn html_to_markdown_strips_remaining_tags() {
        let md = html_to_markdown("<span>hello</span>");
        assert_eq!(md.trim(), "hello");
    }
}
