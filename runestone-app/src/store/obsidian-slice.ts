import type { StateCreator } from 'zustand'
import type { ObsidianImportResult } from '../lib/types'
import * as api from '../lib/api'

export interface ObsidianSlice {
  importResult: ObsidianImportResult | null
  importObsidian: (path: string) => Promise<void>
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export const createObsidianSlice: StateCreator<any, [], [], ObsidianSlice> = (set, get) => ({
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
