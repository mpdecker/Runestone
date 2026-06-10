import type { StateCreator } from 'zustand'
import type { AppStore } from './index'
import * as api from '../lib/api'

export interface ClipperSlice {
  clipperPort: number | null
  clipperLoading: boolean
  clipperError: string | null
  clipperAuthToken: string | null
  loadClipperStatus: () => Promise<void>
  startClipper: () => Promise<void>
  stopClipper: () => Promise<void>
}

export const createClipperSlice: StateCreator<AppStore, [], [], ClipperSlice> = (set, get) => ({
  clipperPort: null,
  clipperLoading: false,
  clipperError: null,
  clipperAuthToken: null,

  loadClipperStatus: async () => {
    try {
      const [port, token] = await Promise.all([
        api.getClipperStatus(),
        api.getClipperAuthToken(),
      ])
      set({ clipperPort: port, clipperAuthToken: token })
    } catch {
      // clipper not available
    }
  },

  startClipper: async () => {
    const { selectedVaultId } = get()
    if (!selectedVaultId) return
    set({ clipperLoading: true })
    try {
      const port = await api.startClipperServer(selectedVaultId)
      const token = await api.getClipperAuthToken()
      set({ clipperPort: port, clipperAuthToken: token, clipperLoading: false, clipperError: null })
    } catch (e) {
      set({ clipperLoading: false, clipperError: `Failed to start clipper: ${e}` })
    }
  },

  stopClipper: async () => {
    try {
      await api.stopClipperServer()
      set({ clipperPort: null, clipperError: null })
    } catch (e) {
      set({ clipperError: `Failed to stop clipper: ${e}` })
    }
  },
})
