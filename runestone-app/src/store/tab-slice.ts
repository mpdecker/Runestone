import type { StateCreator } from 'zustand'

export interface Tab {
  id: string
  title: string
}

export interface TabSlice {
  openTabs: Tab[]
  activeTabId: string | null
  secondaryTabId: string | null
  addTab: (id: string, title: string) => void
  closeTab: (id: string) => void
  switchToTab: (id: string) => void
  closeAllTabs: () => void
  setSecondaryTab: (id: string | null) => void
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export const createTabSlice: StateCreator<any, [], [], TabSlice> = (set, get) => ({
  openTabs: [],
  activeTabId: null,
  secondaryTabId: null,

  addTab: (id: string, title: string) => {
    const { openTabs } = get()
    const existing = openTabs.find((t: Tab) => t.id === id)
    if (!existing) {
      set({ openTabs: [...openTabs, { id, title }], activeTabId: id })
    } else {
      set({ activeTabId: id })
    }
  },

  closeTab: (id: string) => {
    const { openTabs, activeTabId } = get()
    const newTabs = openTabs.filter((t: Tab) => t.id !== id)

    let newActiveId: string | null = null
    if (activeTabId === id) {
      const idx = openTabs.findIndex((t: Tab) => t.id === id)
      if (newTabs.length > 0) {
        newActiveId = newTabs[Math.min(idx, newTabs.length - 1)].id
      }
    } else {
      newActiveId = activeTabId
    }

    set({ openTabs: newTabs, activeTabId: newActiveId })
    if (newActiveId && newActiveId !== activeTabId) {
      get().selectNode?.(newActiveId)
    }
    if (!newActiveId) {
      set({ selectedNodeId: null, currentNode: null, nodeProperties: [], nodeTags: null })
    }
  },

  switchToTab: (id: string) => {
    set({ activeTabId: id })
    get().selectNode?.(id)
  },

  closeAllTabs: () => {
    set({ openTabs: [], activeTabId: null, selectedNodeId: null, currentNode: null })
  },

  setSecondaryTab: (id: string | null) => {
    set({ secondaryTabId: id })
    if (id) {
      get().selectNode?.(id)
    }
  },
})
