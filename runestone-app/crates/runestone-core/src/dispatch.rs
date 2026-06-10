use crate::context::BackendContext;
use crate::error::AppError;
use crate::handlers::{ai, composer, document, embeddings, extraction, graph, node, properties, search, tag, templates, vault, versions};
use crate::models::chat::ChatRequest;
use crate::models::graph::GraphOptions;
use crate::models::node::{CreateNodeRequest, ListNodesRequest, UpdateNodeRequest};
use crate::models::properties::SetPropertyRequest;
use crate::models::search::SearchQuery;
use crate::models::tag::{AddTagsRequest, RemoveTagRequest};
use crate::models::vault::CreateVaultRequest;
use uuid::Uuid;

pub const DESKTOP_ONLY_COMMANDS: &[&str] = &[
    "start_clipper_server",
    "stop_clipper_server",
    "get_clipper_status",
    "get_clipper_auth_token",
    "scan_vault",
    "import_document",
    "import_obsidian_vault",
    "export_node_to_markdown",
    "export_vault_to_markdown",
    "list_available_plugins",
    "read_plugin_file",
];

pub fn is_desktop_only(command: &str) -> bool {
    DESKTOP_ONLY_COMMANDS.contains(&command)
}

pub async fn dispatch_local(
    ctx: &BackendContext,
    command: &str,
    args: serde_json::Value,
) -> Result<serde_json::Value, String> {
    if is_desktop_only(command) {
        return Err(AppError::DesktopOnly(command.to_string()).to_string());
    }

    match command {
        "init_database" => {
            let result = vault::init_database(ctx).await?;
            Ok(serde_json::Value::String(result))
        }
        "create_vault" => {
            let request: CreateVaultRequest =
                serde_json::from_value(args).map_err(|e| e.to_string())?;
            let result = vault::create_vault(ctx, request).await?;
            serde_json::to_value(result).map_err(|e| e.to_string())
        }
        "list_vaults" => {
            let result = vault::list_vaults(ctx).await?;
            serde_json::to_value(result).map_err(|e| e.to_string())
        }
        "create_node" => {
            let request: CreateNodeRequest =
                serde_json::from_value(args).map_err(|e| e.to_string())?;
            let result = node::create_node(ctx, request).await?;
            serde_json::to_value(result).map_err(|e| e.to_string())
        }
        "update_node" => {
            let request: UpdateNodeRequest =
                serde_json::from_value(args).map_err(|e| e.to_string())?;
            let result = node::update_node(ctx, request).await?;
            serde_json::to_value(result).map_err(|e| e.to_string())
        }
        "delete_node" => {
            let id: Uuid = parse_uuid_arg(args)?;
            node::delete_node(ctx, id).await?;
            Ok(serde_json::Value::Null)
        }
        "get_node" => {
            let id: Uuid = parse_uuid_arg(args)?;
            let result = node::get_node(ctx, id).await?;
            serde_json::to_value(result).map_err(|e| e.to_string())
        }
        "list_nodes" => {
            let request: ListNodesRequest = if args.is_string() || args.get("vault_id").is_none() {
                ListNodesRequest {
                    vault_id: parse_uuid_arg(args)?,
                    limit: None,
                    offset: None,
                }
            } else {
                serde_json::from_value(args).map_err(|e| e.to_string())?
            };
            let result = node::list_nodes(ctx, request).await?;
            serde_json::to_value(result).map_err(|e| e.to_string())
        }
        "reindex_vault" => {
            let vault_id: Uuid = parse_uuid_arg(args)?;
            let result = embeddings::reindex_vault(ctx, vault_id).await?;
            Ok(serde_json::json!({ "enqueued": result }))
        }
        "get_embedding_status" => {
            let result = embeddings::get_embedding_status(ctx).await?;
            serde_json::to_value(result).map_err(|e| e.to_string())
        }
        "get_random_node" => {
            let vault_id: Uuid = parse_uuid_arg(args)?;
            let result = node::get_random_node(ctx, vault_id).await?;
            serde_json::to_value(result).map_err(|e| e.to_string())
        }
        "get_graph_data" => {
            let vault_id: Uuid = parse_field(&args, "vault_id")?;
            let options: Option<GraphOptions> = parse_optional_field(&args, "options")?;
            let result = graph::get_graph_data(ctx, vault_id, options).await?;
            serde_json::to_value(result).map_err(|e| e.to_string())
        }
        "get_local_graph" => {
            let node_id: Uuid = parse_field(&args, "node_id")?;
            let depth: Option<u32> = parse_optional_field(&args, "depth")?;
            let result = graph::get_local_graph(ctx, node_id, depth).await?;
            serde_json::to_value(result).map_err(|e| e.to_string())
        }
        "parse_wiki_links" => {
            let node_id: Uuid = parse_uuid_arg(args)?;
            let result = graph::parse_wiki_links(ctx, node_id).await?;
            serde_json::to_value(result).map_err(|e| e.to_string())
        }
        "get_backlinks" => {
            let node_id: Uuid = parse_uuid_arg(args)?;
            let result = graph::get_backlinks(ctx, node_id).await?;
            serde_json::to_value(result).map_err(|e| e.to_string())
        }
        "get_outgoing_links" => {
            let node_id: Uuid = parse_uuid_arg(args)?;
            let result = graph::get_outgoing_links(ctx, node_id).await?;
            serde_json::to_value(result).map_err(|e| e.to_string())
        }
        "semantic_search" => {
            let query: SearchQuery = serde_json::from_value(args).map_err(|e| e.to_string())?;
            let result = search::semantic_search(ctx, query).await?;
            serde_json::to_value(result).map_err(|e| e.to_string())
        }
        "find_similar" => {
            let node_id: Uuid = parse_field(&args, "node_id")?;
            let limit: Option<i64> = parse_optional_field(&args, "limit")?;
            let result = search::find_similar(ctx, node_id, limit).await?;
            serde_json::to_value(result).map_err(|e| e.to_string())
        }
        "hybrid_search" => {
            let query: SearchQuery = serde_json::from_value(args).map_err(|e| e.to_string())?;
            let result = search::hybrid_search(ctx, query).await?;
            serde_json::to_value(result).map_err(|e| e.to_string())
        }
        "boolean_search" => {
            let query: SearchQuery = serde_json::from_value(args).map_err(|e| e.to_string())?;
            let result = search::boolean_search(ctx, query).await?;
            serde_json::to_value(result).map_err(|e| e.to_string())
        }
        "regex_search" => {
            let vault_id: Uuid = parse_field(&args, "vault_id")?;
            let pattern: String = parse_field(&args, "pattern")?;
            let case_sensitive: Option<bool> = parse_optional_field(&args, "case_sensitive")?;
            let limit: Option<i64> = parse_optional_field(&args, "limit")?;
            let result =
                search::regex_search(ctx, vault_id, pattern, case_sensitive, limit).await?;
            serde_json::to_value(result).map_err(|e| e.to_string())
        }
        "get_node_by_alias" => {
            let vault_id: Uuid = parse_field(&args, "vault_id")?;
            let alias: String = parse_field(&args, "alias")?;
            let result = search::get_node_by_alias(ctx, vault_id, alias).await?;
            serde_json::to_value(result).map_err(|e| e.to_string())
        }
        "add_alias" => {
            let node_id: Uuid = parse_field(&args, "node_id")?;
            let alias: String = parse_field(&args, "alias")?;
            let result = search::add_alias(ctx, node_id, alias).await?;
            Ok(result)
        }
        "remove_alias" => {
            let node_id: Uuid = parse_field(&args, "node_id")?;
            let alias: String = parse_field(&args, "alias")?;
            let result = search::remove_alias(ctx, node_id, alias).await?;
            Ok(result)
        }
        "extract_from_document" => {
            let node_id: Uuid = parse_uuid_arg(args)?;
            let result = document::extract_from_document(ctx, node_id).await?;
            serde_json::to_value(result).map_err(|e| e.to_string())
        }
        "get_pending_extractions" => {
            let vault_id: Uuid = parse_uuid_arg(args)?;
            let result = extraction::get_pending_extractions(ctx, vault_id).await?;
            serde_json::to_value(result).map_err(|e| e.to_string())
        }
        "approve_extraction" => {
            let extraction_id: Uuid = parse_uuid_arg(args)?;
            extraction::approve_extraction(ctx, extraction_id).await?;
            Ok(serde_json::Value::Null)
        }
        "reject_extraction" => {
            let extraction_id: Uuid = parse_uuid_arg(args)?;
            extraction::reject_extraction(ctx, extraction_id).await?;
            Ok(serde_json::Value::Null)
        }
        "batch_approve_extractions" => {
            let extraction_ids: Vec<Uuid> =
                serde_json::from_value(args).map_err(|e| e.to_string())?;
            extraction::batch_approve_extractions(ctx, extraction_ids).await?;
            Ok(serde_json::Value::Null)
        }
        "summarize_node" => {
            let node_id: Uuid = parse_uuid_arg(args)?;
            let result = ai::summarize_node(ctx, node_id).await?;
            Ok(serde_json::Value::String(result))
        }
        "chat_with_graph" => {
            let request: ChatRequest = serde_json::from_value(args).map_err(|e| e.to_string())?;
            let result = ai::chat_with_graph(ctx, request).await?;
            serde_json::to_value(result).map_err(|e| e.to_string())
        }
        "suggest_tags" => {
            let node_id: Uuid = parse_uuid_arg(args)?;
            let result = ai::suggest_tags(ctx, node_id).await?;
            serde_json::to_value(result).map_err(|e| e.to_string())
        }
        "get_node_tags" => {
            let node_id: Uuid = parse_uuid_arg(args)?;
            let result = tag::get_node_tags(ctx, node_id).await?;
            serde_json::to_value(result).map_err(|e| e.to_string())
        }
        "add_tags_to_node" => {
            let request: AddTagsRequest =
                serde_json::from_value(args).map_err(|e| e.to_string())?;
            let result = tag::add_tags_to_node(ctx, request).await?;
            serde_json::to_value(result).map_err(|e| e.to_string())
        }
        "remove_tag_from_node" => {
            let request: RemoveTagRequest =
                serde_json::from_value(args).map_err(|e| e.to_string())?;
            let result = tag::remove_tag_from_node(ctx, request).await?;
            serde_json::to_value(result).map_err(|e| e.to_string())
        }
        "list_tags" => {
            let vault_id: Uuid = parse_uuid_arg(args)?;
            let result = tag::list_tags(ctx, vault_id).await?;
            serde_json::to_value(result).map_err(|e| e.to_string())
        }
        "get_nodes_by_tag" => {
            let vault_id: Uuid = parse_field(&args, "vault_id")?;
            let tag_name: String = parse_field(&args, "tag")?;
            let result = tag::get_nodes_by_tag(ctx, vault_id, tag_name).await?;
            serde_json::to_value(result).map_err(|e| e.to_string())
        }
        "accept_tag_suggestions" => {
            let node_id: Uuid = parse_field(&args, "node_id")?;
            let tags: Vec<String> = parse_field(&args, "tags")?;
            let result = tag::accept_tag_suggestions(ctx, node_id, tags).await?;
            serde_json::to_value(result).map_err(|e| e.to_string())
        }
        "get_node_properties" => {
            let node_id: Uuid = parse_uuid_arg(args)?;
            let result = properties::get_node_properties(ctx, node_id).await?;
            serde_json::to_value(result).map_err(|e| e.to_string())
        }
        "set_node_property" => {
            let request: SetPropertyRequest =
                serde_json::from_value(args).map_err(|e| e.to_string())?;
            let result = properties::set_node_property(ctx, request).await?;
            serde_json::to_value(result).map_err(|e| e.to_string())
        }
        "remove_node_property" => {
            let node_id: Uuid = parse_field(&args, "node_id")?;
            let key: String = parse_field(&args, "key")?;
            let result = properties::remove_node_property(ctx, node_id, key).await?;
            serde_json::to_value(result).map_err(|e| e.to_string())
        }
        "create_daily_note" => {
            let vault_id: Uuid = parse_uuid_arg(args)?;
            let result = templates::create_daily_note(ctx, vault_id).await?;
            serde_json::to_value(result).map_err(|e| e.to_string())
        }
        "list_templates" => {
            let vault_id: Uuid = parse_uuid_arg(args)?;
            let result = templates::list_templates(ctx, vault_id).await?;
            serde_json::to_value(result).map_err(|e| e.to_string())
        }
        "create_node_from_template" => {
            let template_id: Uuid = parse_field(&args, "template_id")?;
            let title: Option<String> = parse_optional_field(&args, "title")?;
            let result = templates::create_node_from_template(ctx, template_id, title).await?;
            serde_json::to_value(result).map_err(|e| e.to_string())
        }
        "get_node_versions" => {
            let node_id: Uuid = parse_uuid_arg(args)?;
            let result = versions::get_node_versions(ctx, node_id).await?;
            serde_json::to_value(result).map_err(|e| e.to_string())
        }
        "restore_node_version" => {
            let version_id: Uuid = parse_uuid_arg(args)?;
            let result = versions::restore_node_version(ctx, version_id).await?;
            serde_json::to_value(result).map_err(|e| e.to_string())
        }
        "merge_nodes" => {
            let source_id: Uuid = parse_field(&args, "source_id")?;
            let target_id: Uuid = parse_field(&args, "target_id")?;
            let result = composer::merge_nodes(ctx, source_id, target_id).await?;
            serde_json::to_value(result).map_err(|e| e.to_string())
        }
        "split_node" => {
            let node_id: Uuid = parse_field(&args, "node_id")?;
            let new_title: String = parse_field(&args, "new_title")?;
            let result = composer::split_node(ctx, node_id, new_title).await?;
            serde_json::to_value(result).map_err(|e| e.to_string())
        }
        other => Err(format!("Unknown command: {}", other)),
    }
}

fn parse_uuid_arg(args: serde_json::Value) -> Result<Uuid, String> {
    if let Some(s) = args.as_str() {
        return Uuid::parse_str(s).map_err(|e| e.to_string());
    }
    serde_json::from_value(args).map_err(|e| e.to_string())
}

fn parse_field<T: serde::de::DeserializeOwned>(
    args: &serde_json::Value,
    field: &str,
) -> Result<T, String> {
    if let Some(obj) = args.as_object() {
        if let Some(val) = obj.get(field) {
            return serde_json::from_value(val.clone()).map_err(|e| e.to_string());
        }
    }
    serde_json::from_value(args.clone()).map_err(|e| e.to_string())
}

fn parse_optional_field<T: serde::de::DeserializeOwned>(
    args: &serde_json::Value,
    field: &str,
) -> Result<Option<T>, String> {
    if let Some(obj) = args.as_object() {
        if let Some(val) = obj.get(field) {
            if val.is_null() {
                return Ok(None);
            }
            return serde_json::from_value(val.clone())
                .map(Some)
                .map_err(|e| e.to_string());
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn desktop_only_commands_listed() {
        assert!(is_desktop_only("scan_vault"));
        assert!(is_desktop_only("import_document"));
        assert!(!is_desktop_only("create_node"));
    }
}
