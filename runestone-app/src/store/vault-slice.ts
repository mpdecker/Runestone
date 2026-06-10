import type { StateCreator } from 'zustand'
import type { Vault } from '../lib/types'
import type { AppStore } from './index'
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

export const createVaultSlice: StateCreator<AppStore, [], [], VaultSlice> = (set, get) => ({
  vaults: [],
  selectedVaultId: null,
  isLoading: false,
  error: null,

  initDb: async () => {
    try {
      const status = await api.getConnectionStatus()
      if (!status.connected && status.mode.startsWith('remote')) {
        set({ error: 'Remote server not connected. Test the connection before continuing.' })
        return
      }
      await api.initDatabase()
    } catch (e) {
      console.error('Database init failed:', e)
      set({ error: `Database init failed: ${e}` })
    }
  },

  loadVaults: async () => {
    set({ isLoading: true, error: null })
    try {
      const status = await api.getConnectionStatus()
      if (!status.connected && status.mode.startsWith('remote')) {
        set({
          error: 'Remote server not connected. Test the connection before loading vaults.',
          isLoading: false,
        })
        return
      }
      const vaults = await api.listVaults()
      set({ vaults, isLoading: false })
    } catch (e) {
      set({ error: `Failed to load vaults: ${e}`, isLoading: false })
    }
  },

  createVault: async (name: string, rootPath: string) => {
    set({ isLoading: true, error: null })
    try {
      const status = await api.getConnectionStatus()
      if (!status.connected && status.mode.startsWith('remote')) {
        set({
          error: 'Remote server not connected. Test the connection before creating a vault.',
          isLoading: false,
        })
        return
      }
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
    api.stopVaultWatcher().catch(() => {})
    api.startVaultWatcher(vaultId).catch((e) => console.warn('Vault watcher:', e))
  },
})
