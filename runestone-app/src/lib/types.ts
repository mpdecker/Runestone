export interface Vault {
  id: string
  name: string
  root_path: string
  created_at: string | null
  updated_at: string | null
}

export interface Node {
  id: string
  vault_id: string
  title: string
  content: string
  content_type: string
  file_path: string | null
  metadata: unknown
  word_count: number
  created_at: string | null
  updated_at: string | null
}

export interface NodeListItem {
  id: string
  title: string
  content_type: string
  file_path: string | null
  updated_at: string | null
}

export interface ScanVaultResult {
  created: number
  updated: number
  skipped: number
  deleted: number
}

export interface EmbeddingStatus {
  pending: number
  processing: number
  completed: number
  failed: number
}

export interface CreateVaultRequest {
  name: string
  root_path: string
}

export interface CreateNodeRequest {
  vault_id: string
  title: string
  content: string
  content_type?: string
  file_path?: string
}

export interface UpdateNodeRequest {
  id: string
  title?: string
  content?: string
  content_type?: string
}

export interface GraphNode {
  id: string
  title: string
  content_type: string
}

export interface GraphEdge {
  source: string
  target: string
  label: string
}

export interface GraphData {
  nodes: GraphNode[]
  edges: GraphEdge[]
}

export interface Backlink {
  node_id: string
  title: string
  content_type: string
  context?: string | null
}

export interface GraphOptions {
  depth?: number
  tag?: string
}

export interface WikiLinkRow {
  id: string
  source_node_id: string
  target_title: string
  resolved_node_id: string | null
  context: string | null
  created_at: string | null
}

export interface SearchResult {
  node_id: string
  title: string
  content_type: string
  snippet: string
  score: number
}

export interface SearchResults {
  vector_results: SearchResult[]
  fts_results: SearchResult[]
  combined: SearchResult[]
}

export interface SearchQuery {
  vault_id: string
  query: string
  limit?: number
  include_fts?: boolean
}

export interface ExtractionNode {
  name: string
  description: string
  extraction_type: string
  confidence: number
  source_node_id: string
  chunk_index: number
}

export interface PendingExtraction {
  id: string
  title: string
  content_type: string
  metadata: {
    status: string
    confidence: number
    source_chunk: number
    description: string
    extraction_type: string
  } | null
  created_at: string | null
}

export interface ChatMessage {
  role: string
  content: string
}

export interface ChatRequest {
  vault_id: string
  question: string
  history: ChatMessage[]
}

export interface ChatResponse {
  answer: string
  citations: Citation[]
}

export interface Citation {
  node_id: string
  title: string
  snippet: string
}

export interface TagSuggestion {
  name: string
  confidence: number
  reason: string
}

export interface ObsidianImportResult {
  nodes_created: number
  links_created: number
  files_scanned: number
}

export interface TagsResponse {
  node_id: string
  tags: string[]
}

export interface TagInfo {
  name: string
  node_count: number | null
}

export interface AddTagsRequest {
  node_id: string
  tags: string[]
}

export interface RemoveTagRequest {
  node_id: string
  tag: string
}

export interface NodeProperty {
  key: string
  value: unknown
  prop_type: string
}

export interface PropertiesResponse {
  node_id: string
  properties: NodeProperty[]
}

export interface SetPropertyRequest {
  node_id: string
  key: string
  value: unknown
}

export interface NodeVersion {
  id: string
  node_id: string
  version_number: number
  title: string
  content: string
  word_count: number
  created_at: string | null
}
