mod commands;
mod desktop_dispatch;
mod router;
mod remote_api;
mod state;
mod vault_watcher;

pub use runestone_core::{
    db, document, embedding, error, llm, models, path_guard, repositories, services, util,
};

use state::AppState;

#[tauri::command]
fn get_platform() -> String {
    if cfg!(target_os = "ios") {
        "ios".into()
    } else if cfg!(target_os = "android") {
        "android".into()
    } else if cfg!(target_os = "windows") {
        "windows".into()
    } else if cfg!(target_os = "macos") {
        "macos".into()
    } else {
        "linux".into()
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            AppState::init(app).map_err(|e| e.to_string().into())
        })
        .invoke_handler(tauri::generate_handler![
            get_platform,
            commands::server::configure_server_connection,
            commands::server::get_connection_status,
            commands::server::test_connection,
            commands::vault::init_database,
            commands::vault::create_vault,
            commands::vault::list_vaults,
            commands::node::create_node,
            commands::node::update_node,
            commands::node::delete_node,
            commands::node::get_node,
            commands::node::list_nodes,
            commands::node::scan_vault,
            commands::node::get_random_node,
            commands::graph::get_graph_data,
            commands::graph::get_local_graph,
            commands::graph::parse_wiki_links,
            commands::graph::get_backlinks,
            commands::graph::get_outgoing_links,
            commands::search::semantic_search,
            commands::search::find_similar,
            commands::search::hybrid_search,
            commands::search::boolean_search,
            commands::search::regex_search,
            commands::search::get_node_by_alias,
            commands::search::add_alias,
            commands::search::remove_alias,
            commands::document::import_document,
            commands::document::extract_from_document,
            commands::extraction::get_pending_extractions,
            commands::extraction::approve_extraction,
            commands::extraction::reject_extraction,
            commands::extraction::batch_approve_extractions,
            commands::ai::summarize_node,
            commands::ai::chat_with_graph,
            commands::ai::chat_with_graph_stream,
            commands::ai::suggest_tags,
            commands::vault::start_vault_watcher,
            commands::vault::stop_vault_watcher,
            commands::embeddings::reindex_vault,
            commands::embeddings::get_embedding_status,
            commands::obsidian::import_obsidian_vault,
            commands::tag::get_node_tags,
            commands::tag::add_tags_to_node,
            commands::tag::remove_tag_from_node,
            commands::tag::list_tags,
            commands::tag::get_nodes_by_tag,
            commands::tag::accept_tag_suggestions,
            commands::properties::get_node_properties,
            commands::properties::set_node_property,
            commands::properties::remove_node_property,
            commands::export::export_node_to_markdown,
            commands::export::export_vault_to_markdown,
            commands::templates::create_daily_note,
            commands::templates::list_templates,
            commands::templates::create_node_from_template,
            commands::versions::get_node_versions,
            commands::versions::restore_node_version,
            commands::composer::merge_nodes,
            commands::composer::split_node,
            commands::clipper::start_clipper_server,
            commands::clipper::stop_clipper_server,
            commands::clipper::get_clipper_status,
            commands::clipper::get_clipper_auth_token,
            commands::plugins::list_available_plugins,
            commands::plugins::read_plugin_file,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
