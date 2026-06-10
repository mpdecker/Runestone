import { describe, it, expect, beforeEach, vi } from 'vitest'
import { useStore } from '@/store'

const mockApi = vi.hoisted(() => ({
  hybridSearch: vi.fn().mockResolvedValue({
    vector_results: [],
    fts_results: [],
    combined: [{ node_id: 'n-1', title: 'Hit', content_type: 'note', snippet: '...', score: 0.9 }],
  }),
  findSimilar: vi.fn().mockResolvedValue([{ node_id: 'n-2', title: 'Similar', content_type: 'note', snippet: '...', score: 0.8 }]),
}))

vi.mock('@/lib/api', () => ({ ...mockApi }))

function resetStore() {
  useStore.setState({
    selectedVaultId: 'v-1',
    searchQuery: 'test query',
    searchResults: null,
    searchLoading: false,
    searchError: null,
    showSearch: false,
    similarNodes: [],
  })
}

describe('search-slice', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    resetStore()
  })

  it('runSearch stores hybrid results', async () => {
    await useStore.getState().runSearch()
    expect(mockApi.hybridSearch).toHaveBeenCalled()
    expect(useStore.getState().searchResults?.combined).toHaveLength(1)
    expect(useStore.getState().showSearch).toBe(true)
  })

  it('findSimilar stores similar nodes', async () => {
    await useStore.getState().findSimilar('n-1')
    expect(useStore.getState().similarNodes).toHaveLength(1)
  })

  it('toggleSearch flips visibility', () => {
    useStore.getState().toggleSearch()
    expect(useStore.getState().showSearch).toBe(true)
    useStore.getState().toggleSearch()
    expect(useStore.getState().showSearch).toBe(false)
  })
})
