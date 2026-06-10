import { describe, it, expect, beforeEach, vi } from 'vitest'
import { useStore } from '@/store'

const mockApi = vi.hoisted(() => ({
  getClipperStatus: vi.fn().mockResolvedValue(null),
  getClipperAuthToken: vi.fn().mockResolvedValue('token-123'),
  startClipperServer: vi.fn().mockResolvedValue(8765),
  stopClipperServer: vi.fn().mockResolvedValue(undefined),
}))

vi.mock('@/lib/api', () => ({ ...mockApi }))

function resetStore() {
  useStore.setState({
    selectedVaultId: 'v-1',
    clipperPort: null,
    clipperLoading: false,
    clipperError: null,
    clipperAuthToken: null,
  })
}

describe('clipper-slice', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    resetStore()
  })

  it('startClipper sets port and token', async () => {
    await useStore.getState().startClipper()
    expect(useStore.getState().clipperPort).toBe(8765)
    expect(useStore.getState().clipperAuthToken).toBe('token-123')
  })

  it('stopClipper clears port', async () => {
    useStore.setState({ clipperPort: 8765 })
    await useStore.getState().stopClipper()
    expect(useStore.getState().clipperPort).toBeNull()
  })

  it('loadClipperStatus reads token', async () => {
    await useStore.getState().loadClipperStatus()
    expect(useStore.getState().clipperAuthToken).toBe('token-123')
  })
})
