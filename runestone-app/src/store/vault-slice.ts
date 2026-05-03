import type { StateCreator } from 'zustand'
import type { Vault } from '../lib/types'
import * as api from '../lib/api'

export interface VaultSlice {
  vaults: Vault[]
  selectedVaultId: string | null
  isLoading: boolean
  error: string | null
  loadVaults: () => Promise<void>
  createVault: (name: string, rootPath: string) => Promise<void>
  selectVault: (vaultId: string) => void
  initDb: () => Promise<void>
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export const createVaultSlice: StateCreator<any, [], [], VaultSlice> = (set, get) => ({
  vaults: [],
  selectedVaultId: null,
  isLoading: false,
  error: null,

  initDb: async () => {
    try {
      await api.initDatabase()
    } catch (e) {
      console.error('Database init failed:', e)
      set({ error: `Database init failed: ${e}` })
    }
  },

  loadVaults: async () => {
    set({ isLoading: true, error: null })
    try {
      const vaults = await api.listVaults()
      set({ vaults, isLoading: false })
    } catch (e) {
      set({ error: `Failed to load vaults: ${e}`, isLoading: false })
    }
  },

  createVault: async (name: string, rootPath: string) => {
    set({ isLoading: true, error: null })
    try {
      await api.createVault({ name, root_path: rootPath })
      await get().loadVaults()
    } catch (e) {
      set({ error: `Failed to create vault: ${e}`, isLoading: false })
    }
  },

  selectVault: (vaultId: string) => {
    set({ selectedVaultId: vaultId, selectedNodeId: null, currentNode: null, nodes: [], graphData: null, nodeTags: null, selectedTag: null })
    get().loadNodes()
    get().loadGraphData()
    get().loadVaultTags?.()
  },
})
