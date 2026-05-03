import type { StateCreator } from 'zustand'

export interface UISlice {
  darkMode: boolean
  showCommandPalette: boolean
  sidebarCollapsed: boolean
  filterText: string
  filterTypes: string[]
  listViewMode: 'list' | 'tree'
  readingMode: boolean
  splitMode: 'off' | 'vertical'
  userCss: string
  toggleDarkMode: () => void
  toggleCommandPalette: () => void
  toggleChat: () => void
  toggleSidebar: () => void
  setFilterText: (text: string) => void
  toggleFilterType: (type: string) => void
  setListViewMode: (mode: 'list' | 'tree') => void
  toggleReadingMode: () => void
  toggleSplitMode: () => void
  setUserCss: (css: string) => void
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export const createUISlice: StateCreator<any, [], [], UISlice> = (set, get) => ({
  darkMode: loadPersisted('darkMode', false),
  showCommandPalette: false,
  sidebarCollapsed: loadPersisted('sidebarCollapsed', false),
  filterText: '',
  filterTypes: [],
  listViewMode: loadPersisted<'list' | 'tree'>('listViewMode', 'list'),
  readingMode: loadPersisted('readingMode', false),
  splitMode: 'off',
  userCss: loadPersisted('userCss', ''),

  toggleDarkMode: () => {
    set((s: { darkMode: boolean }) => {
      const next = !s.darkMode
      document.documentElement.classList.toggle('dark', next)
      persist('darkMode', next)
      return { darkMode: next }
    })
  },

  toggleCommandPalette: () => {
    set((s: { showCommandPalette: boolean }) => ({ showCommandPalette: !s.showCommandPalette }))
  },

  toggleChat: () => {
    set((s: { showChat: boolean }) => ({ showChat: !s.showChat }))
  },

  toggleSidebar: () => {
    set((s: { sidebarCollapsed: boolean }) => {
      const next = !s.sidebarCollapsed
      persist('sidebarCollapsed', next)
      return { sidebarCollapsed: next }
    })
  },

  setFilterText: (text: string) => {
    set({ filterText: text })
  },

  toggleFilterType: (type: string) => {
    const { filterTypes } = get()
    if (filterTypes.includes(type)) {
      set({ filterTypes: filterTypes.filter((t: string) => t !== type) })
    } else {
      set({ filterTypes: [...filterTypes, type] })
    }
  },

  setListViewMode: (mode: 'list' | 'tree') => {
    persist('listViewMode', mode)
    set({ listViewMode: mode })
  },

  toggleReadingMode: () => {
    set((s: { readingMode: boolean }) => {
      const next = !s.readingMode
      persist('readingMode', next)
      return { readingMode: next }
    })
  },

  toggleSplitMode: () => {
    set((s: { splitMode: string }) => ({
      splitMode: s.splitMode === 'vertical' ? 'off' : 'vertical'
    }))
  },

  setUserCss: (css: string) => {
    persist('userCss', css)
    set({ userCss: css })
  },
})

function loadPersisted<T>(key: string, fallback: T): T {
  try {
    const raw = localStorage.getItem(`runestone_${key}`)
    if (raw === null) return fallback
    return JSON.parse(raw) as T
  } catch {
    return fallback
  }
}

function persist(key: string, value: unknown) {
  try {
    localStorage.setItem(`runestone_${key}`, JSON.stringify(value))
  } catch {
    // storage full
  }
}
