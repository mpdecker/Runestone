import { describe, it, expect, beforeEach, vi } from 'vitest'
import { useStore } from '@/store'

const mockApi = vi.hoisted(() => ({
  getConnectionStatus: vi.fn().mockResolvedValue({ connected: true, mode: 'local' }),
  initDatabase: vi.fn().mockResolvedValue('ok'),
  listVaults: vi.fn().mockResolvedValue([{ id: 'v-1', name: 'Test', root_path: '/vault' }]),
  createVault: vi.fn().mockResolvedValue({ id: 'v-2', name: 'New', root_path: '/new' }),
  listNodes: vi.fn().mockResolvedValue([]),
  getGraphData: vi.fn().mockResolvedValue({ nodes: [], edges: [] }),
  stopVaultWatcher: vi.fn().mockResolvedValue(undefined),
  startVaultWatcher: vi.fn().mockResolvedValue(undefined),
}))

vi.mock('@/lib/api', () => ({ ...mockApi }))

function resetStore() {
  useStore.setState({
    vaults: [],
    selectedVaultId: null,
    isLoading: false,
    error: null,
    nodes: [],
  })
}

describe('vault-slice', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    resetStore()
  })

  it('loadVaults fetches vaults', async () => {
    await useStore.getState().loadVaults()
    expect(mockApi.listVaults).toHaveBeenCalled()
    expect(useStore.getState().vaults).toHaveLength(1)
    expect(useStore.getState().isLoading).toBe(false)
  })

  it('loadVaults sets error when remote disconnected', async () => {
    mockApi.getConnectionStatus.mockResolvedValueOnce({ connected: false, mode: 'remote' })
    await useStore.getState().loadVaults()
    expect(useStore.getState().error).toContain('Remote server not connected')
  })

  it('selectVault loads nodes and starts watcher', async () => {
    useStore.setState({ selectedVaultId: 'v-1' })
    useStore.getState().selectVault('v-1')
    expect(useStore.getState().selectedVaultId).toBe('v-1')
    expect(mockApi.startVaultWatcher).toHaveBeenCalledWith('v-1')
  })
})
