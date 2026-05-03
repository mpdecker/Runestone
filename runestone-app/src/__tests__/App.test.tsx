import { describe, it, expect, beforeEach, vi } from 'vitest'
import { render, screen, waitFor } from '@testing-library/react'
import App from '@/App'
import { useStore } from '@/store'

vi.mock('@/features/graph', () => ({
  GraphCanvas: () => <div data-testid="graph-canvas">Graph</div>,
}))

vi.mock('@/features/sidebar', () => ({
  Sidebar: () => <div data-testid="sidebar">Sidebar</div>,
}))

vi.mock('@/features/editor', () => ({
  NoteEditor: () => <div data-testid="note-editor">Editor</div>,
}))

vi.mock('@/features/search', () => ({
  SearchPanel: () => <div data-testid="search-panel">Search</div>,
}))

vi.mock('@/features/chat', () => ({
  ChatPanel: () => <div data-testid="chat-panel">Chat</div>,
}))

vi.mock('@/features/extraction', () => ({
  ExtractionReview: () => <div data-testid="extraction-review">Extractions</div>,
}))

vi.mock('@/features/command-palette', () => ({
  CommandPalette: () => <div data-testid="command-palette">Commands</div>,
}))

vi.mock('@/lib/api', () => ({
  initDatabase: vi.fn().mockResolvedValue('ok'),
  listVaults: vi.fn().mockResolvedValue([]),
}))

function resetStore() {
  useStore.setState({
    vaults: [],
    selectedVaultId: null,
    nodes: [],
    selectedNodeId: null,
    currentNode: null,
    isEditorDirty: false,
    isLoading: false,
    error: null,
    graphData: null,
    graphViewMode: 'global',
    backlinks: [],
    showBacklinks: false,
    filterText: '',
    filterTypes: [],
    searchQuery: '',
    searchResults: null,
    searchLoading: false,
    showSearch: false,
    similarNodes: [],
    pendingExtractions: [],
    showExtractions: false,
    extractionResults: [],
    darkMode: false,
    nodeSummary: null,
    summaryLoading: false,
    suggestedLinks: [],
    showCommandPalette: false,
    showChat: false,
    chatMessages: [],
    chatLoading: false,
    chatCitations: [],
    chatAnswer: null,
    tagSuggestions: [],
    importResult: null,
  })
}

async function waitForDesktop() {
  await waitFor(() => {
    expect(screen.getByTestId('sidebar')).toBeInTheDocument()
  })
}

describe('App', () => {
  beforeEach(() => {
    resetStore()
  })

  it('renders sidebar and graph canvas', async () => {
    render(<App />)
    await waitForDesktop()
    expect(screen.getByTestId('graph-canvas')).toBeInTheDocument()
  })

  it('renders NoteEditor when node selected', async () => {
    useStore.setState({ selectedNodeId: 'n-1' })
    render(<App />)
    await waitFor(() => {
      expect(screen.getByTestId('note-editor')).toBeInTheDocument()
    })
  })

  it('does not render NoteEditor when no node selected', async () => {
    render(<App />)
    await waitForDesktop()
    expect(screen.queryByTestId('note-editor')).not.toBeInTheDocument()
  })

  it('renders all panels', async () => {
    render(<App />)
    await waitForDesktop()
    expect(screen.getByTestId('search-panel')).toBeInTheDocument()
    expect(screen.getByTestId('chat-panel')).toBeInTheDocument()
    expect(screen.getByTestId('extraction-review')).toBeInTheDocument()
    expect(screen.getByTestId('command-palette')).toBeInTheDocument()
  })
})
