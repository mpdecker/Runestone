import type { StateCreator } from 'zustand'
import type { SearchResults, SearchResult } from '../lib/types'
import type { AppStore } from './index'
import * as api from '../lib/api'

export interface SearchSlice {
  searchQuery: string
  searchResults: SearchResults | null
  searchLoading: boolean
  searchError: string | null
  showSearch: boolean
  similarNodes: SearchResult[]
  setSearchQuery: (query: string) => void
  runSearch: () => Promise<void>
  findSimilar: (nodeId: string) => Promise<void>
  toggleSearch: () => void
}

export const createSearchSlice: StateCreator<AppStore, [], [], SearchSlice> = (set, get) => ({
  searchQuery: '',
  searchResults: null,
  searchLoading: false,
  searchError: null,
  showSearch: false,
  similarNodes: [],

  setSearchQuery: (query: string) => {
    set({ searchQuery: query })
  },

  runSearch: async () => {
    const { selectedVaultId, searchQuery } = get()
    if (!selectedVaultId || !searchQuery.trim()) return
    set({ searchLoading: true, searchError: null })
    try {
      const results = await api.hybridSearch({
        vault_id: selectedVaultId,
        query: searchQuery,
        limit: 20,
        include_fts: true,
      })
      set({ searchResults: results, searchLoading: false, showSearch: true })
    } catch (e) {
      set({ searchError: `Search failed: ${e}`, searchLoading: false })
    }
  },

  findSimilar: async (nodeId: string) => {
    set({ searchLoading: true, searchError: null })
    try {
      const results = await api.findSimilar(nodeId, 10)
      set({ similarNodes: results, searchLoading: false, showSearch: true })
    } catch (e) {
      set({ searchError: `Find similar failed: ${e}`, searchLoading: false })
    }
  },

  toggleSearch: () => {
    set((s) => ({ showSearch: !s.showSearch }))
  },
})
