import { describe, it, expect, beforeEach, vi } from 'vitest'
import { useStore } from '@/store'

const mockApi = vi.hoisted(() => ({
  initDatabase: vi.fn().mockResolvedValue('ok'),
  listVaults: vi.fn().mockResolvedValue([]),
  getNode: vi.fn().mockResolvedValue({ id: 'n-1', vault_id: 'v-1', title: 'Test', content: '<p>Hi</p>', content_type: 'note', file_path: null, metadata: {}, word_count: 1, created_at: null, updated_at: null }),
  listNodes: vi.fn().mockResolvedValue([]),
  loadGraphData: vi.fn().mockResolvedValue({ nodes: [], edges: [] }),
}))

vi.mock('@/lib/api', () => ({ ...mockApi }))

function resetStore() {
  useStore.setState({
    vaults: [],
    selectedVaultId: null,
    nodes: [],
    selectedNodeId: null,
    currentNode: null,
    openTabs: [],
    activeTabId: null,
    secondaryTabId: null,
    nodeProperties: [],
    nodeTags: null,
    graphData: null,
    error: null,
    isLoading: false,
  })
}

describe('tab-slice', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    resetStore()
  })

  describe('addTab', () => {
    it('adds a new tab and sets it as active', () => {
      useStore.getState().addTab('n-1', 'Node One')

      const { openTabs, activeTabId } = useStore.getState()
      expect(openTabs).toHaveLength(1)
      expect(openTabs[0]).toEqual({ id: 'n-1', title: 'Node One' })
      expect(activeTabId).toBe('n-1')
    })

    it('does not duplicate existing tab but sets it active', () => {
      useStore.getState().addTab('n-1', 'Node One')
      useStore.getState().addTab('n-2', 'Node Two')
      useStore.getState().addTab('n-1', 'Node One')

      const { openTabs } = useStore.getState()
      expect(openTabs).toHaveLength(2)
      expect(openTabs.map((t) => t.id)).toEqual(['n-1', 'n-2'])
    })

    it('moves active tab to existing tab', () => {
      useStore.getState().addTab('n-1', 'First')
      useStore.getState().addTab('n-2', 'Second')
      useStore.getState().addTab('n-1', 'First')

      expect(useStore.getState().activeTabId).toBe('n-1')
    })
  })

  describe('closeTab', () => {
    it('removes the tab from openTabs', () => {
      useStore.getState().addTab('n-1', 'First')
      useStore.getState().addTab('n-2', 'Second')
      useStore.getState().closeTab('n-1')

      const { openTabs } = useStore.getState()
      expect(openTabs).toHaveLength(1)
      expect(openTabs[0].id).toBe('n-2')
    })

    it('selects adjacent tab when closing active tab', () => {
      useStore.getState().addTab('n-1', 'First')
      useStore.getState().addTab('n-2', 'Second')
      useStore.getState().addTab('n-3', 'Third')

      useStore.setState({ activeTabId: 'n-2' })
      useStore.getState().closeTab('n-2')

      expect(useStore.getState().activeTabId).toBe('n-3')
    })

    it('clears node state when closing last tab', () => {
      useStore.setState({ selectedNodeId: 'n-1', currentNode: { id: 'n-1' }, nodeTags: { node_id: 'n-1', tags: [] } })
      useStore.getState().addTab('n-1', 'First')
      useStore.getState().closeTab('n-1')

      expect(useStore.getState().openTabs).toEqual([])
      expect(useStore.getState().activeTabId).toBeNull()
      expect(useStore.getState().selectedNodeId).toBeNull()
      expect(useStore.getState().currentNode).toBeNull()
    })

    it('keeps active tab when closing non-active tab', () => {
      useStore.getState().addTab('n-1', 'First')
      useStore.getState().addTab('n-2', 'Second')
      useStore.setState({ activeTabId: 'n-1' })

      useStore.getState().closeTab('n-2')

      expect(useStore.getState().activeTabId).toBe('n-1')
    })
  })

  describe('switchToTab', () => {
    it('sets activeTabId', () => {
      useStore.getState().addTab('n-1', 'First')
      useStore.getState().addTab('n-2', 'Second')
      useStore.setState({ activeTabId: 'n-1' })

      useStore.getState().switchToTab('n-2')

      expect(useStore.getState().activeTabId).toBe('n-2')
    })
  })

  describe('closeAllTabs', () => {
    it('clears all tabs and resets node state', () => {
      useStore.setState({ selectedNodeId: 'n-1', currentNode: { id: 'n-1' } })
      useStore.getState().addTab('n-1', 'First')
      useStore.getState().addTab('n-2', 'Second')

      useStore.getState().closeAllTabs()

      const { openTabs, activeTabId, selectedNodeId, currentNode } = useStore.getState()
      expect(openTabs).toEqual([])
      expect(activeTabId).toBeNull()
      expect(selectedNodeId).toBeNull()
      expect(currentNode).toBeNull()
    })
  })

  describe('setSecondaryTab', () => {
    it('sets secondaryTabId', () => {
      useStore.getState().setSecondaryTab('n-2')

      expect(useStore.getState().secondaryTabId).toBe('n-2')
    })

    it('clears secondaryTabId with null', () => {
      useStore.getState().setSecondaryTab('n-2')
      useStore.getState().setSecondaryTab(null)

      expect(useStore.getState().secondaryTabId).toBeNull()
    })
  })
})
