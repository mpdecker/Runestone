import type { StateCreator } from 'zustand'
import type { PendingExtraction, ExtractionNode } from '../lib/types'
import * as api from '../lib/api'

export interface ExtractionSlice {
  pendingExtractions: PendingExtraction[]
  showExtractions: boolean
  extractionResults: ExtractionNode[]
  importDocument: (filePath: string) => Promise<void>
  extractFromDocument: (nodeId: string) => Promise<void>
  loadPendingExtractions: () => Promise<void>
  approveExtraction: (id: string) => Promise<void>
  rejectExtraction: (id: string) => Promise<void>
  batchApproveExtractions: (ids: string[]) => Promise<void>
  toggleExtractions: () => void
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export const createExtractionSlice: StateCreator<any, [], [], ExtractionSlice> = (set, get) => ({
  pendingExtractions: [],
  showExtractions: false,
  extractionResults: [],

  importDocument: async (filePath: string) => {
    const { selectedVaultId } = get()
    if (!selectedVaultId) return
    set({ isLoading: true, error: null })
    try {
      await api.importDocument(selectedVaultId, filePath)
      await get().loadNodes()
      await get().loadGraphData()
      set({ isLoading: false })
    } catch (e) {
      set({ error: `Import failed: ${e}`, isLoading: false })
    }
  },

  extractFromDocument: async (nodeId: string) => {
    set({ isLoading: true, error: null })
    try {
      const results = await api.extractFromDocument(nodeId)
      set({ extractionResults: results, isLoading: false })
      await get().loadNodes()
      await get().loadGraphData()
    } catch (e) {
      set({ error: `Extraction failed: ${e}`, isLoading: false })
    }
  },

  loadPendingExtractions: async () => {
    const { selectedVaultId } = get()
    if (!selectedVaultId) return
    try {
      const extractions = await api.getPendingExtractions(selectedVaultId)
      set({ pendingExtractions: extractions })
    } catch (e) {
      console.error('Failed to load pending extractions:', e)
    }
  },

  approveExtraction: async (id: string) => {
    try {
      await api.approveExtraction(id)
      set((s: { pendingExtractions: PendingExtraction[] }) => ({ pendingExtractions: s.pendingExtractions.filter((p: PendingExtraction) => p.id !== id) }))
    } catch (e) {
      set({ error: `Approve failed: ${e}` })
    }
  },

  rejectExtraction: async (id: string) => {
    try {
      await api.rejectExtraction(id)
      set((s: { pendingExtractions: PendingExtraction[] }) => ({ pendingExtractions: s.pendingExtractions.filter((p: PendingExtraction) => p.id !== id) }))
    } catch (e) {
      set({ error: `Reject failed: ${e}` })
    }
  },

  batchApproveExtractions: async (ids: string[]) => {
    try {
      await api.batchApproveExtractions(ids)
      set((s: { pendingExtractions: PendingExtraction[] }) => ({ pendingExtractions: s.pendingExtractions.filter((p: PendingExtraction) => !ids.includes(p.id)) }))
    } catch (e) {
      set({ error: `Batch approve failed: ${e}` })
    }
  },

  toggleExtractions: () => {
    const { showExtractions } = get()
    if (!showExtractions) {
      get().loadPendingExtractions()
    }
    set({ showExtractions: !showExtractions })
  },
})
