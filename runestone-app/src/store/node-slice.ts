import { listen } from '@tauri-apps/api/event'
import type { StateCreator } from 'zustand'
import type { Node, NodeListItem } from '../lib/types'
import type { AppStore } from './index'
import * as api from '../lib/api'

let vaultWatcherUnlisten: (() => void) | null = null

export async function setupVaultFileWatcher(
  reload: () => Promise<void>,
): Promise<void> {
  if (typeof window === 'undefined' || !('__TAURI_INTERNALS__' in window)) {
    return
  }
  if (vaultWatcherUnlisten) {
    vaultWatcherUnlisten()
    vaultWatcherUnlisten = null
  }
  try {
    vaultWatcherUnlisten = await listen('vault-file-changed', () => {
      reload()
    })
  } catch {
    // Tauri event API unavailable (e.g. vitest)
  }
}

export interface NodeSlice {
  nodes: NodeListItem[]
  selectedNodeId: string | null
  currentNode: Node | null
  secondaryNode: Node | null
  nodeError: string | null
  nodeLoading: boolean
  isEditorDirty: boolean
  isSecondaryEditorDirty: boolean
  isSaving: boolean
  isSecondarySaving: boolean
  wikiLinkPreviews: Record<string, string>
  loadNodes: () => Promise<void>
  loadNodesByTag: (tag: string) => Promise<void>
  scanVault: () => Promise<void>
  selectNode: (nodeId: string) => Promise<void>
  selectSecondaryNode: (nodeId: string) => Promise<void>
  createNode: (title: string) => Promise<void>
  updateNodeContent: (content: string, secondary?: boolean) => void
  saveNode: (secondary?: boolean) => Promise<void>
  deleteNode: (id: string) => Promise<void>
  fetchWikiLinkPreview: (nodeId: string) => Promise<string | null>
}

export const createNodeSlice: StateCreator<AppStore, [], [], NodeSlice> = (set, get) => ({
  nodes: [],
  selectedNodeId: null,
  currentNode: null,
  secondaryNode: null,
  nodeError: null,
  nodeLoading: false,
  isEditorDirty: false,
  isSecondaryEditorDirty: false,
  isSaving: false,
  isSecondarySaving: false,
  wikiLinkPreviews: {},

  loadNodes: async () => {
    const { selectedVaultId } = get()
    if (!selectedVaultId) return
    set({ nodeLoading: true, nodeError: null })
    try {
      const nodes = await api.listNodes(selectedVaultId)
      set({ nodes, nodeLoading: false })
    } catch (e) {
      set({ nodeError: `Failed to load nodes: ${e}`, nodeLoading: false })
    }
  },

  loadNodesByTag: async (tag: string) => {
    const { selectedVaultId } = get()
    if (!selectedVaultId) return
    set({ nodeLoading: true, nodeError: null })
    try {
      const nodes = await api.getNodesByTag(selectedVaultId, tag)
      set({ nodes, nodeLoading: false })
    } catch (e) {
      set({ nodeError: `Failed to load nodes by tag: ${e}`, nodeLoading: false })
    }
  },

  scanVault: async () => {
    const { selectedVaultId } = get()
    if (!selectedVaultId) return
    set({ nodeLoading: true, nodeError: null })
    try {
      await api.scanVault(selectedVaultId, true)
      await get().loadNodes()
      await get().loadGraphData()
      set({ nodeLoading: false })
    } catch (e) {
      set({ nodeError: `Failed to scan vault: ${e}`, nodeLoading: false })
    }
  },

  selectNode: async (nodeId: string) => {
    set({ nodeLoading: true, nodeError: null, isEditorDirty: false, showBacklinks: false, backlinks: [] })
    try {
      const node = await api.getNode(nodeId)
      set({ selectedNodeId: nodeId, currentNode: node, nodeLoading: false })
      get().addTab?.(nodeId, node.title)
      get().loadNodeTags?.(nodeId)
      get().loadNodeProperties?.(nodeId)
      get().loadNodeVersions?.(nodeId)
    } catch (e) {
      set({ nodeError: `Failed to load node: ${e}`, nodeLoading: false })
    }
  },

  selectSecondaryNode: async (nodeId: string) => {
    try {
      const node = await api.getNode(nodeId)
      set({ secondaryNode: node, secondaryTabId: nodeId })
      get().loadNodeTags?.(nodeId)
    } catch (e) {
      console.error('Failed to load secondary node:', e)
    }
  },

  createNode: async (title: string) => {
    const { selectedVaultId } = get()
    if (!selectedVaultId) return
    set({ nodeLoading: true, nodeError: null })
    try {
      const node = await api.createNode({
        vault_id: selectedVaultId,
        title,
        content: '',
      })
      await get().loadNodes()
      await get().loadGraphData()
      set({ selectedNodeId: node.id, currentNode: node, nodeLoading: false, isEditorDirty: false })
    } catch (e) {
      set({ nodeError: `Failed to create node: ${e}`, nodeLoading: false })
    }
  },

  updateNodeContent: (content: string, secondary = false) => {
    if (secondary) {
      const { secondaryNode } = get()
      if (!secondaryNode) return
      set({
        secondaryNode: { ...secondaryNode, content },
        isSecondaryEditorDirty: true,
      })
      return
    }
    const { currentNode } = get()
    if (!currentNode) return
    set({ currentNode: { ...currentNode, content }, isEditorDirty: true })
  },

  saveNode: async (secondary = false) => {
    const node = secondary ? get().secondaryNode : get().currentNode
    if (!node) return
    if (secondary) {
      set({ isSecondarySaving: true, nodeError: null })
    } else {
      set({ isSaving: true, nodeError: null })
    }
    try {
      const updated = await api.updateNode({
        id: node.id,
        content: node.content,
      })
      if (secondary) {
        set({
          secondaryNode: updated,
          isSecondarySaving: false,
          isSecondaryEditorDirty: false,
        })
      } else {
        set({ currentNode: updated, isSaving: false, isEditorDirty: false })
      }
    } catch (e) {
      if (secondary) {
        set({ nodeError: `Failed to save note: ${e}`, isSecondarySaving: false })
      } else {
        set({ nodeError: `Failed to save node: ${e}`, isSaving: false })
      }
    }
  },

  deleteNode: async (id: string) => {
    set({ nodeLoading: true, nodeError: null })
    try {
      await api.deleteNode(id)
      set({ selectedNodeId: null, currentNode: null, nodeLoading: false, isEditorDirty: false })
      await get().loadNodes()
      await get().loadGraphData()
    } catch (e) {
      set({ nodeError: `Failed to delete node: ${e}`, nodeLoading: false })
    }
  },

  fetchWikiLinkPreview: async (nodeId: string) => {
    const cached = get().wikiLinkPreviews[nodeId]
    if (cached) return cached
    try {
      const fullNode = await api.getNode(nodeId)
      const snippet = fullNode.content
        .replace(/<[^>]+>/g, '')
        .replace(/\s+/g, ' ')
        .trim()
        .slice(0, 250)
      set({
        wikiLinkPreviews: { ...get().wikiLinkPreviews, [nodeId]: snippet },
      })
      return snippet
    } catch {
      return null
    }
  },
})
