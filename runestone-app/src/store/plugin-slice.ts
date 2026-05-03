import type { StateCreator } from 'zustand'
import type { PluginInfo, PluginInstance } from '../lib/plugin-types'
import * as api from '../lib/api'
import { getPluginManager, type RegisteredPanel, type RegisteredCommand } from '../lib/plugin-manager'

export interface PluginSlice {
  pluginDir: string
  availablePlugins: PluginInfo[]
  installedPlugins: PluginInstance[]
  pluginLoading: boolean
  registeredPanels: RegisteredPanel[]
  registeredCommands: RegisteredCommand[]
  setPluginDir: (dir: string) => void
  discoverPlugins: () => Promise<void>
  loadPlugin: (info: PluginInfo) => Promise<void>
  togglePlugin: (name: string) => Promise<void>
  unloadPlugin: (name: string) => void
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export const createPluginSlice: StateCreator<any, [], [], PluginSlice> = (set, get) => ({
  pluginDir: '',
  availablePlugins: [],
  installedPlugins: [],
  pluginLoading: false,
  registeredPanels: [],
  registeredCommands: [],

  setPluginDir: (dir: string) => {
    set({ pluginDir: dir })
  },

  discoverPlugins: async () => {
    const { pluginDir } = get()
    if (!pluginDir) return
    set({ pluginLoading: true })
    try {
      const plugins = await api.listAvailablePlugins(pluginDir)
      set({ availablePlugins: plugins, pluginLoading: false })
    } catch (e) {
      console.error('Failed to discover plugins:', e)
      set({ pluginLoading: false })
    }
  },

  loadPlugin: async (info: PluginInfo) => {
    const mgr = getPluginManager()
    if (mgr.isLoaded(info.name)) return

    try {
      const mainPath = `${info.path.replace(/\\/g, '/')}/${info.main_file}`
      const code = await api.readPluginFile(mainPath)
      await mgr.loadPlugin(info, code)
      const plugins = mgr.getPlugins()
      set({ installedPlugins: plugins })
    } catch (e) {
      console.error('Failed to load plugin:', e)
    }
  },

  togglePlugin: async (name: string) => {
    const mgr = getPluginManager()
    const plugin = mgr.getPlugin(name)
    if (!plugin) return

    if (plugin.enabled) {
      await mgr.deactivatePlugin(name)
    } else {
      const store = get()
      const apiModule = api
      const hooks = mgr.hook
      mgr.setAPI({
        store,
        api: apiModule,
        hooks,
        registerSidebarPanel: (id: string, render: (container: HTMLElement) => void) => {
          const mgr = getPluginManager()
          mgr.registerPanel(id, name, render)
          set({ registeredPanels: mgr.getPanels() })
        },
        registerCommand: (id: string, label: string, handler: () => void) => {
          const mgr = getPluginManager()
          mgr.registerCommand(id, name, label, handler)
          set({ registeredCommands: mgr.getCommands() })
        },
      })
      await mgr.activatePlugin(name)
    }

    set({ installedPlugins: mgr.getPlugins() })
  },

  unloadPlugin: (name: string) => {
    getPluginManager().deactivatePlugin(name)
    set({ installedPlugins: getPluginManager().getPlugins() })
  },
})
