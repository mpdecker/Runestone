/**
 * IPC bindings — incrementally generated from Rust models via specta.
 * Regenerate with: pnpm generate:bindings
 */

export interface ScanVaultResult {
  created: number
  updated: number
  skipped: number
  deleted: number
}

export interface ListNodesRequest {
  vault_id: string
  limit?: number
  offset?: number
}

export interface EmbeddingJobStatus {
  pending: number
  processing: number
  completed: number
  failed: number
}

export interface SearchResultProvenance {
  node_id: string
  title: string
  score: number
  source: 'semantic' | 'fts' | 'both'
}
