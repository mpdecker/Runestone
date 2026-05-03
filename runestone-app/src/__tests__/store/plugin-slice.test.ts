import { describe, it, expect, vi, beforeEach } from 'vitest'
import { useStore } from '@/store'

const mockApi = vi.hoisted(() => ({
  initDatabase: vi.fn().mockResolvedValue('ok'),
  listVaults: vi.fn().mockResolvedValue([]),
  listAvailablePlugins: vi.fn().mockResolvedValue([]),
  readPluginFile: vi.fn().mockResolvedValue('// plugin code'),
}))

const mockManager = {
  plugins: new Map<string, { manifest: { name: string }; enabled: boolean; error?: string }>(),
  isLoaded: (name: string) => mockManager.plugins.has(name),
  loadPlugin: vi.fn(async (info: { name: string }) => {
    mockManager.plugins.set(info.name, { manifest: { name: info.name }, enabled: false })
  }),
  getPlugins: () => Array.from(mockManager.plugins.values()),
  getPlugin: (name: string) => mockManager.plugins.get(name) || null,
  activatePlugin: vi.fn(async (name: string) => {
    const p = mockManager.plugins.get(name)
    if (p) p.enabled = true
  }),
  deactivatePlugin: vi.fn(async (name: string) => {
    const p = mockManager.plugins.get(name)
    if (p) p.enabled = false
  }),
  setAPI: vi.fn(),
  hook: { on: vi.fn(), off: vi.fn(), emit: vi.fn() },
}

vi.mock('@/lib/api', () => ({ ...mockApi }))

vi.mock('@/lib/plugin-manager', () => ({
  getPluginManager: vi.fn(() => mockManager),
}))

function resetStore() {
  useStore.setState({
    vaults: [],
    selectedVaultId: null,
    pluginDir: '',
    availablePlugins: [],
    installedPlugins: [],
    pluginLoading: false,
  })
  mockManager.plugins.clear()
}

describe('plugin-slice', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    resetStore()
  })

  describe('setPluginDir', () => {
    it('updates pluginDir in state', () => {
      useStore.getState().setPluginDir('/home/plugins')

      expect(useStore.getState().pluginDir).toBe('/home/plugins')
    })
  })

  describe('discoverPlugins', () => {
    it('does nothing when pluginDir is empty', async () => {
      await useStore.getState().discoverPlugins()

      expect(mockApi.listAvailablePlugins).not.toHaveBeenCalled()
    })

    it('calls API and sets availablePlugins', async () => {
      const plugins = [{ name: 'test-plugin', version: '1.0.0', path: '/p/test', main_file: 'index.js' }]
      mockApi.listAvailablePlugins.mockResolvedValue(plugins)
      useStore.setState({ pluginDir: '/home/plugins' })

      await useStore.getState().discoverPlugins()

      expect(mockApi.listAvailablePlugins).toHaveBeenCalledWith('/home/plugins')
      expect(useStore.getState().availablePlugins).toEqual(plugins)
      expect(useStore.getState().pluginLoading).toBe(false)
    })

    it('sets loading false on error', async () => {
      mockApi.listAvailablePlugins.mockRejectedValue(new Error('fail'))
      const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {})
      useStore.setState({ pluginDir: '/bad' })

      await useStore.getState().discoverPlugins()

      expect(useStore.getState().pluginLoading).toBe(false)
      consoleSpy.mockRestore()
    })
  })

  describe('loadPlugin', () => {
    it('does not reload already loaded plugins', async () => {
      mockManager.plugins.set('loaded', { manifest: { name: 'loaded' }, enabled: false })
      const info = { name: 'loaded', version: '1.0.0', path: '/p', main_file: 'index.js' }

      await useStore.getState().loadPlugin(info)

      expect(mockApi.readPluginFile).not.toHaveBeenCalled()
    })
  })

  describe('togglePlugin', () => {
    it('does nothing when plugin is not found', async () => {
      await useStore.getState().togglePlugin('nonexistent')

      expect(mockManager.activatePlugin).not.toHaveBeenCalled()
      expect(mockManager.deactivatePlugin).not.toHaveBeenCalled()
    })
  })

  describe('unloadPlugin', () => {
    it('calls deactivate on the plugin manager', () => {
      useStore.getState().unloadPlugin('test-plugin')

      expect(mockManager.deactivatePlugin).toHaveBeenCalledWith('test-plugin')
    })
  })
})
