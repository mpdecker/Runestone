import type { StateCreator } from 'zustand'
import type { TagsResponse, TagInfo } from '../lib/types'
import type { AppStore } from './index'
import * as api from '../lib/api'

export interface TagSlice {
  nodeTags: TagsResponse | null
  vaultTags: TagInfo[]
  selectedTag: string | null
  loadNodeTags: (nodeId: string) => Promise<void>
  addTags: (nodeId: string, tags: string[]) => Promise<void>
  removeTag: (nodeId: string, tag: string) => Promise<void>
  acceptSuggestedTags: (nodeId: string, tags: string[]) => Promise<void>
  loadVaultTags: () => Promise<void>
  selectTag: (tag: string | null) => void
}

export const createTagSlice: StateCreator<AppStore, [], [], TagSlice> = (set, get) => ({
  nodeTags: null,
  vaultTags: [],
  selectedTag: null,

  loadNodeTags: async (nodeId: string) => {
    try {
      const response = await api.getNodeTags(nodeId)
      set({ nodeTags: response })
    } catch (e) {
      console.error('Failed to load node tags:', e)
    }
  },

  addTags: async (nodeId: string, tags: string[]) => {
    try {
      const response = await api.addTagsToNode({ node_id: nodeId, tags })
      set({ nodeTags: response })
      await get().loadVaultTags()
    } catch (e) {
      console.error('Failed to add tags:', e)
    }
  },

  removeTag: async (nodeId: string, tag: string) => {
    try {
      const response = await api.removeTagFromNode({ node_id: nodeId, tag })
      set({ nodeTags: response })
      await get().loadVaultTags()
    } catch (e) {
      console.error('Failed to remove tag:', e)
    }
  },

  acceptSuggestedTags: async (nodeId: string, tags: string[]) => {
    try {
      const response = await api.acceptTagSuggestions(nodeId, tags)
      set({ nodeTags: response })
      await get().loadVaultTags()
    } catch (e) {
      console.error('Failed to accept tag suggestions:', e)
    }
  },

  loadVaultTags: async () => {
    const { selectedVaultId } = get()
    if (!selectedVaultId) return
    try {
      const tags = await api.listTags(selectedVaultId)
      set({ vaultTags: tags })
    } catch (e) {
      console.error('Failed to load vault tags:', e)
    }
  },

  selectTag: (tag: string | null) => {
    set({ selectedTag: tag })
    if (tag) {
      get().loadNodesByTag?.(tag)
    } else {
      get().loadNodes?.()
    }
  },
})
