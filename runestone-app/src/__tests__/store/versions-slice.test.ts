import { describe, it, expect, beforeEach, vi } from 'vitest'
import { useStore } from '@/store'

const mockApi = vi.hoisted(() => ({
  getNodeVersions: vi.fn().mockResolvedValue([{ id: 'ver-1', node_id: 'n-1', title: 'Old', created_at: null }]),
  restoreNodeVersion: vi.fn().mockResolvedValue(undefined),
  getNode: vi.fn().mockResolvedValue({ id: 'n-1', vault_id: 'v-1', title: 'Restored', content: '<p>x</p>', content_type: 'note', file_path: null, metadata: {}, word_count: 1, created_at: null, updated_at: null }),
  listNodes: vi.fn().mockResolvedValue([]),
}))

vi.mock('@/lib/api', () => ({ ...mockApi }))

function resetStore() {
  useStore.setState({
    selectedNodeId: 'n-1',
    nodeVersions: [],
    versionsLoading: false,
    currentNode: null,
    nodeError: null,
    nodeLoading: false,
  })
}

describe('versions-slice', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    resetStore()
  })

  it('loadNodeVersions stores versions', async () => {
    await useStore.getState().loadNodeVersions('n-1')
    expect(useStore.getState().nodeVersions).toHaveLength(1)
    expect(useStore.getState().versionsLoading).toBe(false)
  })

  it('restoreVersion reloads node', async () => {
    await useStore.getState().restoreVersion('ver-1')
    expect(mockApi.restoreNodeVersion).toHaveBeenCalledWith('ver-1')
    expect(mockApi.getNode).toHaveBeenCalledWith('n-1')
  })
})
