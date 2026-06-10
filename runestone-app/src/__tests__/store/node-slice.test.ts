import { describe, it, expect, beforeEach, vi } from 'vitest'
import { useStore } from '@/store'

const mockApi = vi.hoisted(() => ({
  listNodes: vi.fn().mockResolvedValue([{ id: 'n-1', title: 'Note', content_type: 'note', file_path: null, updated_at: null }]),
  getNode: vi.fn().mockResolvedValue({ id: 'n-1', vault_id: 'v-1', title: 'Note', content: '<p>Hi</p>', content_type: 'note', file_path: null, metadata: {}, word_count: 1, created_at: null, updated_at: null }),
  createNode: vi.fn().mockResolvedValue({ id: 'n-2', vault_id: 'v-1', title: 'New', content: '', content_type: 'note', file_path: null, metadata: {}, word_count: 0, created_at: null, updated_at: null }),
  scanVault: vi.fn().mockResolvedValue({ created: 1, updated: 0, skipped: 0, deleted: 0 }),
  getGraphData: vi.fn().mockResolvedValue({ nodes: [], edges: [] }),
  addTab: vi.fn(),
}))

vi.mock('@/lib/api', () => ({ ...mockApi }))

function resetStore() {
  useStore.setState({
    selectedVaultId: 'v-1',
    nodes: [],
    selectedNodeId: null,
    currentNode: null,
    nodeError: null,
    nodeLoading: false,
    openTabs: [],
    activeTabId: null,
  })
}

describe('node-slice', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    resetStore()
  })

  it('loadNodes populates nodes list', async () => {
    await useStore.getState().loadNodes()
    expect(useStore.getState().nodes).toHaveLength(1)
    expect(useStore.getState().nodeLoading).toBe(false)
  })

  it('createNode selects new node', async () => {
    await useStore.getState().createNode('New')
    expect(mockApi.createNode).toHaveBeenCalled()
    expect(useStore.getState().selectedNodeId).toBe('n-2')
  })

  it('scanVault reloads nodes', async () => {
    await useStore.getState().scanVault()
    expect(mockApi.scanVault).toHaveBeenCalledWith('v-1', true)
    expect(mockApi.listNodes).toHaveBeenCalled()
  })
})
