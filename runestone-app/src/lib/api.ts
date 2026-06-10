import { invoke } from '@tauri-apps/api/core'
import type {
  Vault,
  Node,
  NodeListItem,
  ScanVaultResult,
  EmbeddingStatus,
  CreateVaultRequest,
  CreateNodeRequest,
  UpdateNodeRequest,
  GraphData,
  GraphOptions,
  Backlink,
  WikiLinkRow,
  SearchResult,
  SearchResults,
  SearchQuery,
  ExtractionNode,
  PendingExtraction,
  ChatRequest,
  ChatResponse,
  TagSuggestion,
  ObsidianImportResult,
  TagsResponse,
  TagInfo,
  AddTagsRequest,
  RemoveTagRequest,
  PropertiesResponse,
  SetPropertyRequest,
  NodeVersion,
} from './types'
import type { PluginInfo } from './plugin-types'

export interface ConnectionStatus {
  mode: string
  api_url: string | null
  connected: boolean
  local_db_available: boolean
}

export async function getConnectionStatus(): Promise<ConnectionStatus> {
  return invoke('get_connection_status')
}

export async function testConnection(): Promise<boolean> {
  return invoke('test_connection')
}

export async function initDatabase(): Promise<string> {
  return invoke('init_database')
}

export async function createVault(request: CreateVaultRequest): Promise<Vault> {
  return invoke('create_vault', { request })
}

export async function listVaults(): Promise<Vault[]> {
  return invoke('list_vaults')
}

export async function createNode(request: CreateNodeRequest): Promise<Node> {
  return invoke('create_node', { request })
}

export async function updateNode(request: UpdateNodeRequest): Promise<Node> {
  return invoke('update_node', { request })
}

export async function deleteNode(id: string): Promise<void> {
  return invoke('delete_node', { id })
}

export async function getNode(id: string): Promise<Node> {
  return invoke('get_node', { id })
}

export async function listNodes(
  vaultId: string,
  limit?: number,
  offset?: number,
): Promise<NodeListItem[]> {
  return invoke('list_nodes', { vaultId, limit, offset })
}

export async function scanVault(
  vaultId: string,
  deleteOrphans?: boolean,
): Promise<ScanVaultResult> {
  return invoke('scan_vault', { vaultId, deleteOrphans })
}

export async function startVaultWatcher(vaultId: string): Promise<void> {
  return invoke('start_vault_watcher', { vaultId })
}

export async function stopVaultWatcher(): Promise<void> {
  return invoke('stop_vault_watcher')
}

export async function chatWithGraphStream(request: ChatRequest): Promise<ChatResponse> {
  return invoke('chat_with_graph_stream', { request })
}

export async function reindexVault(vaultId: string): Promise<{ enqueued: number }> {
  return invoke('reindex_vault', { vaultId })
}

export async function getEmbeddingStatus(): Promise<EmbeddingStatus> {
  return invoke('get_embedding_status')
}

export async function getRandomNode(vaultId: string): Promise<Node> {
  return invoke('get_random_node', { vaultId })
}

export async function getGraphData(vaultId: string, options?: GraphOptions): Promise<GraphData> {
  return invoke('get_graph_data', { vaultId, options })
}

export async function getLocalGraph(nodeId: string, depth?: number): Promise<GraphData> {
  return invoke('get_local_graph', { nodeId, depth })
}

export async function parseWikiLinks(nodeId: string): Promise<WikiLinkRow[]> {
  return invoke('parse_wiki_links', { nodeId })
}

export async function getBacklinks(nodeId: string): Promise<Backlink[]> {
  return invoke('get_backlinks', { nodeId })
}

export async function getOutgoingLinks(nodeId: string): Promise<Backlink[]> {
  return invoke('get_outgoing_links', { nodeId })
}

export async function semanticSearch(query: SearchQuery): Promise<SearchResult[]> {
  return invoke('semantic_search', { query })
}

export async function findSimilar(nodeId: string, limit?: number): Promise<SearchResult[]> {
  return invoke('find_similar', { nodeId, limit })
}

export async function hybridSearch(query: SearchQuery): Promise<SearchResults> {
  return invoke('hybrid_search', { query })
}

export async function booleanSearch(query: SearchQuery): Promise<SearchResult[]> {
  return invoke('boolean_search', { query })
}

export async function regexSearch(vaultId: string, pattern: string, caseSensitive?: boolean, limit?: number): Promise<SearchResult[]> {
  return invoke('regex_search', { vaultId, pattern, caseSensitive, limit })
}

export async function getNodeByAlias(vaultId: string, alias: string): Promise<Node | null> {
  return invoke('get_node_by_alias', { vaultId, alias })
}

export async function addAlias(nodeId: string, alias: string): Promise<unknown> {
  return invoke('add_alias', { nodeId, alias })
}

export async function removeAlias(nodeId: string, alias: string): Promise<unknown> {
  return invoke('remove_alias', { nodeId, alias })
}

export async function getNodeVersions(nodeId: string): Promise<NodeVersion[]> {
  return invoke('get_node_versions', { nodeId })
}

export async function restoreNodeVersion(versionId: string): Promise<Node> {
  return invoke('restore_node_version', { versionId })
}

export async function mergeNodes(sourceId: string, targetId: string): Promise<Node> {
  return invoke('merge_nodes', { sourceId, targetId })
}

export async function splitNode(nodeId: string, newTitle: string): Promise<[Node, Node]> {
  return invoke('split_node', { nodeId, newTitle })
}

export async function startClipperServer(vaultId: string, port?: number): Promise<number> {
  return invoke('start_clipper_server', { vaultId, port })
}

export async function stopClipperServer(): Promise<void> {
  return invoke('stop_clipper_server')
}

export async function getClipperStatus(): Promise<number | null> {
  return invoke('get_clipper_status')
}

export async function getClipperAuthToken(): Promise<string> {
  return invoke('get_clipper_auth_token')
}

export async function listAvailablePlugins(pluginDir: string): Promise<PluginInfo[]> {
  return invoke('list_available_plugins', { pluginDir })
}

export async function readPluginFile(pluginRoot: string, relativePath: string): Promise<string> {
  return invoke('read_plugin_file', { pluginRoot, relativePath })
}

export async function importDocument(vaultId: string, filePath: string): Promise<Node> {
  return invoke('import_document', { vaultId, filePath })
}

export async function extractFromDocument(nodeId: string): Promise<ExtractionNode[]> {
  return invoke('extract_from_document', { nodeId })
}

export async function getPendingExtractions(vaultId: string): Promise<PendingExtraction[]> {
  return invoke('get_pending_extractions', { vaultId })
}

export async function approveExtraction(extractionId: string): Promise<void> {
  return invoke('approve_extraction', { extractionId })
}

export async function rejectExtraction(extractionId: string): Promise<void> {
  return invoke('reject_extraction', { extractionId })
}

export async function batchApproveExtractions(extractionIds: string[]): Promise<void> {
  return invoke('batch_approve_extractions', { extractionIds })
}

export async function summarizeNode(nodeId: string): Promise<string> {
  return invoke('summarize_node', { nodeId })
}

export async function chatWithGraph(request: ChatRequest): Promise<ChatResponse> {
  return invoke('chat_with_graph', { request })
}

export async function suggestTags(nodeId: string): Promise<TagSuggestion[]> {
  return invoke('suggest_tags', { nodeId })
}

export async function importObsidianVault(vaultId: string, rootPath: string): Promise<ObsidianImportResult> {
  return invoke('import_obsidian_vault', { vaultId, rootPath })
}

export async function getNodeTags(nodeId: string): Promise<TagsResponse> {
  return invoke('get_node_tags', { nodeId })
}

export async function addTagsToNode(request: AddTagsRequest): Promise<TagsResponse> {
  return invoke('add_tags_to_node', { request })
}

export async function removeTagFromNode(request: RemoveTagRequest): Promise<TagsResponse> {
  return invoke('remove_tag_from_node', { request })
}

export async function listTags(vaultId: string): Promise<TagInfo[]> {
  return invoke('list_tags', { vaultId })
}

export async function getNodesByTag(vaultId: string, tag: string): Promise<NodeListItem[]> {
  return invoke('get_nodes_by_tag', { vaultId, tag })
}

export async function acceptTagSuggestions(nodeId: string, tags: string[]): Promise<TagsResponse> {
  return invoke('accept_tag_suggestions', { nodeId, tags })
}

export async function getNodeProperties(nodeId: string): Promise<PropertiesResponse> {
  return invoke('get_node_properties', { nodeId })
}

export async function setNodeProperty(request: SetPropertyRequest): Promise<PropertiesResponse> {
  return invoke('set_node_property', { request })
}

export async function removeNodeProperty(nodeId: string, key: string): Promise<PropertiesResponse> {
  return invoke('remove_node_property', { nodeId, key })
}

export async function exportNodeToMarkdown(nodeId: string, exportPath?: string): Promise<string> {
  return invoke('export_node_to_markdown', { nodeId, exportPath })
}

export async function exportVaultToMarkdown(vaultId: string, outputDir: string): Promise<string[]> {
  return invoke('export_vault_to_markdown', { vaultId, outputDir })
}

export async function createDailyNote(vaultId: string): Promise<Node> {
  return invoke('create_daily_note', { vaultId })
}

export async function listTemplates(vaultId: string): Promise<NodeListItem[]> {
  return invoke('list_templates', { vaultId })
}

export async function createNodeFromTemplate(templateId: string, title?: string): Promise<Node> {
  return invoke('create_node_from_template', { templateId, title })
}
