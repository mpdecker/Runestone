import type { StateCreator } from 'zustand'
import type { ObsidianImportResult } from '../lib/types'
import type { AppStore } from './index'
import * as api from '../lib/api'

export interface ObsidianSlice {
  importResult: ObsidianImportResult | null
  importObsidian: (path: string) => Promise<void>
}

export const createObsidianSlice: StateCreator<AppStore, [], [], ObsidianSlice> = (set, get) => ({
  importResult: null,

  importObsidian: async (path: string) => {
    const { selectedVaultId } = get()
    if (!selectedVaultId) return
    set({ isLoading: true, error: null })
    try {
      const result = await api.importObsidianVault(selectedVaultId, path)
      set({ importResult: result, isLoading: false })
      await get().loadNodes()
      await get().loadGraphData()
    } catch (e) {
      set({ error: `Import failed: ${e}`, isLoading: false })
    }
  },
})
