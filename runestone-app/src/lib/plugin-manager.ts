import type { PluginAPI, PluginInfo, PluginInstance, PluginManifest } from './plugin-types'

export interface RegisteredPanel {
  id: string
  render: (container: HTMLElement) => void
  plugin: string
}

export interface RegisteredCommand {
  id: string
  label: string
  handler: () => void
  plugin: string
}

const plugins: Map<string, PluginInstance> = new Map()
const hookHandlers: Map<string, Set<(...args: unknown[]) => void>> = new Map()
const panels: Map<string, RegisteredPanel> = new Map()
const commands: Map<string, RegisteredCommand> = new Map()

let pluginAPI: PluginAPI | null = null
let onPanelsChange: (() => void) | null = null
let onCommandsChange: (() => void) | null = null

export function getPluginManager() {
  return {
    getPlugins: () => Array.from(plugins.values()),
    getPlugin: (name: string) => plugins.get(name),
    isLoaded: (name: string) => plugins.has(name),
    getPanels: () => Array.from(panels.values()),
    getCommands: () => Array.from(commands.values()),
    onPanelsChange: (cb: () => void) => { onPanelsChange = cb },
    onCommandsChange: (cb: () => void) => { onCommandsChange = cb },

    setAPI: (api: PluginAPI) => {
      pluginAPI = api
    },

    loadPlugin: async (info: PluginInfo, code: string): Promise<PluginInstance> => {
      if (plugins.has(info.name)) {
        return plugins.get(info.name)!
      }

      try {
        const moduleExports: Record<string, unknown> = {}
        const fn = new Function('exports', 'require', code)
        fn(moduleExports, undefined)

        const manifest: PluginManifest = {
          name: info.name,
          version: info.version,
          description: info.description,
          author: info.author || undefined,
          main: info.main_file,
        }

        const instance: PluginInstance = {
          manifest,
          activate: (moduleExports.activate as PluginInstance['activate']) || (() => {}),
          deactivate: (moduleExports.deactivate as PluginInstance['deactivate']) || (() => {}),
          enabled: false,
        }

        plugins.set(info.name, instance)
        return instance
      } catch (e) {
        const instance: PluginInstance = {
          manifest: {
            name: info.name,
            version: info.version,
            description: info.description,
            author: info.author || undefined,
            main: info.main_file,
          },
          activate: () => {},
          deactivate: () => {},
          enabled: false,
          error: String(e),
        }
        plugins.set(info.name, instance)
        return instance
      }
    },

    activatePlugin: async (name: string) => {
      const plugin = plugins.get(name)
      if (!plugin || plugin.enabled) return false

      try {
        if (pluginAPI) {
          await plugin.activate(pluginAPI)
        }
        plugin.enabled = true
        return true
      } catch (e) {
        plugin.error = String(e)
        return false
      }
    },

    deactivatePlugin: async (name: string) => {
      const plugin = plugins.get(name)
      if (!plugin || !plugin.enabled) return false

      try {
        await plugin.deactivate()
        plugin.enabled = false
        pluginAPI = null
        for (const [id, panel] of panels) {
          if (panel.plugin === name) panels.delete(id)
        }
        for (const [id, cmd] of commands) {
          if (cmd.plugin === name) commands.delete(id)
        }
        onPanelsChange?.()
        onCommandsChange?.()
        return true
      } catch (e) {
        plugin.error = String(e)
        return false
      }
    },

    hook: {
      on: (event: string, handler: (...args: unknown[]) => void) => {
        if (!hookHandlers.has(event)) {
          hookHandlers.set(event, new Set())
        }
        hookHandlers.get(event)!.add(handler)
      },
      off: (event: string, handler: (...args: unknown[]) => void) => {
        hookHandlers.get(event)?.delete(handler)
      },
      emit: (event: string, ...args: unknown[]) => {
        hookHandlers.get(event)?.forEach((h) => h(...args))
      },
    },

    registerPanel: (id: string, plugin: string, render: (container: HTMLElement) => void) => {
      panels.set(id, { id, render, plugin })
      onPanelsChange?.()
    },

    registerCommand: (id: string, plugin: string, label: string, handler: () => void) => {
      commands.set(id, { id, label, handler, plugin })
      onCommandsChange?.()
    },

    unregisterPlugin: (plugin: string) => {
      for (const [id, panel] of panels) {
        if (panel.plugin === plugin) panels.delete(id)
      }
      for (const [id, cmd] of commands) {
        if (cmd.plugin === plugin) commands.delete(id)
      }
      onPanelsChange?.()
      onCommandsChange?.()
    },
  }
}
