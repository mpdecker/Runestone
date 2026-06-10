import { describe, it, expect, beforeEach, vi } from 'vitest'
import { useStore } from '@/store'

const mockApi = vi.hoisted(() => ({
  getGraphData: vi.fn().mockResolvedValue({ nodes: [{ id: 'n-1', label: 'A', type: 'note' }], edges: [] }),
  getLocalGraph: vi.fn().mockResolvedValue({ nodes: [], edges: [] }),
  listNodes: vi.fn().mockResolvedValue([]),
}))

vi.mock('@/lib/api', () => ({ ...mockApi }))

function resetStore() {
  useStore.setState({
    selectedVaultId: 'v-1',
    graphData: null,
    graphError: null,
    graphLoading: false,
    graphViewMode: 'global',
  })
}

describe('graph-slice', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    resetStore()
  })

  it('loadGraphData stores graph data', async () => {
    await useStore.getState().loadGraphData()
    expect(useStore.getState().graphData?.nodes).toHaveLength(1)
  })

  it('loadLocalGraph sets local mode and data', async () => {
    await useStore.getState().loadLocalGraph('n-1', 2)
    expect(useStore.getState().graphViewMode).toBe('local')
    expect(mockApi.getLocalGraph).toHaveBeenCalledWith('n-1', 2)
  })

  it('setGraphViewMode global reloads graph', () => {
    useStore.getState().setGraphViewMode('global')
    expect(mockApi.getGraphData).toHaveBeenCalled()
  })
})
