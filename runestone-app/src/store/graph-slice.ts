import type { StateCreator } from 'zustand'
import type { GraphData, Backlink } from '../lib/types'
import * as api from '../lib/api'

export type GraphViewMode = 'global' | 'local'

export interface GraphSlice {
  graphData: GraphData | null
  graphViewMode: GraphViewMode
  graphDepth: number
  backlinks: Backlink[]
  outgoingLinks: Backlink[]
  showBacklinks: boolean
  showOutgoingLinks: boolean
  loadGraphData: (tag?: string) => Promise<void>
  loadLocalGraph: (nodeId: string, depth?: number) => Promise<void>
  setGraphViewMode: (mode: GraphViewMode) => void
  setGraphDepth: (depth: number) => void
  loadBacklinks: (nodeId: string) => Promise<void>
  toggleBacklinks: () => void
  loadOutgoingLinks: (nodeId: string) => Promise<void>
  toggleOutgoingLinks: () => void
  parseWikiLinks: (nodeId: string) => Promise<void>
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export const createGraphSlice: StateCreator<any, [], [], GraphSlice> = (set, get) => ({
  graphData: null,
  graphViewMode: 'global',
  graphDepth: 1,
  backlinks: [],
  outgoingLinks: [],
  showBacklinks: false,
  showOutgoingLinks: false,

  loadGraphData: async (tag?: string) => {
    const { selectedVaultId } = get()
    if (!selectedVaultId) return
    try {
      const options = tag ? { tag } : undefined
      const data = await api.getGraphData(selectedVaultId, options)
      set({ graphData: data })
    } catch (e) {
      console.error('Failed to load graph data:', e)
    }
  },

  loadLocalGraph: async (nodeId: string, depth?: number) => {
    const d = depth ?? get().graphDepth
    set({ graphViewMode: 'local', isLoading: true, graphDepth: d })
    try {
      const data = await api.getLocalGraph(nodeId, d)
      set({ graphData: data, isLoading: false })
    } catch (e) {
      set({ error: `Failed to load local graph: ${e}`, isLoading: false })
    }
  },

  setGraphViewMode: (mode: GraphViewMode) => {
    set({ graphViewMode: mode })
    if (mode === 'global') {
      get().loadGraphData()
    }
  },

  setGraphDepth: (depth: number) => {
    set({ graphDepth: depth })
  },

  loadBacklinks: async (nodeId: string) => {
    try {
      const links = await api.getBacklinks(nodeId)
      set({ backlinks: links })
    } catch (e) {
      console.error('Failed to load backlinks:', e)
    }
  },

  toggleBacklinks: () => {
    const { showBacklinks, selectedNodeId } = get()
    if (!showBacklinks && selectedNodeId) {
      get().loadBacklinks(selectedNodeId)
    }
    set({ showBacklinks: !showBacklinks })
  },

  loadOutgoingLinks: async (nodeId: string) => {
    try {
      const links = await api.getOutgoingLinks(nodeId)
      set({ outgoingLinks: links })
    } catch (e) {
      console.error('Failed to load outgoing links:', e)
    }
  },

  toggleOutgoingLinks: () => {
    const { showOutgoingLinks, selectedNodeId } = get()
    if (!showOutgoingLinks && selectedNodeId) {
      get().loadOutgoingLinks(selectedNodeId)
    }
    set({ showOutgoingLinks: !showOutgoingLinks })
  },

  parseWikiLinks: async (nodeId: string) => {
    try {
      await api.parseWikiLinks(nodeId)
      await get().loadGraphData()
    } catch (e) {
      set({ error: `Failed to parse wiki links: ${e}` })
    }
  },
})
