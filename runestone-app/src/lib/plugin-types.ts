/* eslint-disable @typescript-eslint/no-explicit-any */

export interface PluginManifest {
  name: string
  version: string
  description: string
  author?: string
  main: string
}

export interface PluginAPI {
  store: any
  api: any
  hooks: {
    on: (event: string, handler: (...args: any[]) => void) => void
    off: (event: string, handler: (...args: any[]) => void) => void
    emit: (event: string, ...args: any[]) => void
  }
  registerSidebarPanel: (id: string, render: (container: HTMLElement) => void) => void
  registerCommand: (id: string, label: string, handler: () => void) => void
}

export interface PluginInstance {
  manifest: PluginManifest
  activate: (api: PluginAPI) => void | Promise<void>
  deactivate: () => void | Promise<void>
  enabled: boolean
  error?: string
}

export interface PluginInfo {
  name: string
  version: string
  description: string
  author: string | null
  path: string
  main_file: string
}
