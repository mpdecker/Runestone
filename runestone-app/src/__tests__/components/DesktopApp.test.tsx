import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import { DesktopApp } from '@/features/layout/DesktopApp'
import { useStore } from '@/store'

vi.mock('@/features/sidebar', () => ({
  Sidebar: () => <div data-testid="sidebar">Sidebar</div>,
}))

vi.mock('@/features/graph', () => ({
  GraphCanvas: () => <div data-testid="graph-canvas">Graph</div>,
}))

vi.mock('@/features/editor', () => ({
  NoteEditor: ({ secondary }: { secondary?: boolean }) => (
    <div data-testid={secondary ? 'note-editor-secondary' : 'note-editor'}>
      Editor{secondary ? ' (secondary)' : ''}
    </div>
  ),
}))

vi.mock('@/features/editor/TabBar', () => ({
  TabBar: () => <div data-testid="tab-bar">Tabs</div>,
}))

vi.mock('@/features/search', () => ({
  SearchPanel: () => <div data-testid="search-panel">Search</div>,
}))

vi.mock('@/features/extraction', () => ({
  ExtractionReview: () => <div data-testid="extraction-review">Extractions</div>,
}))

vi.mock('@/features/command-palette', () => ({
  CommandPalette: () => <div data-testid="command-palette">Commands</div>,
}))

vi.mock('@/features/chat', () => ({
  ChatPanel: () => <div data-testid="chat-panel">Chat</div>,
}))

function resetStore() {
  useStore.setState({
    vaults: [],
    selectedVaultId: null,
    nodes: [],
    selectedNodeId: null,
    currentNode: null,
    openTabs: [],
    activeTabId: null,
    secondaryTabId: null,
    secondaryNode: null,
    splitMode: 'off',
    userCss: '',
    nodeProperties: [],
    nodeTags: null,
    graphData: null,
    error: null,
    isLoading: false,
    darkMode: false,
    showCommandPalette: false,
    showSearch: false,
    showExtractions: false,
    showChat: false,
  })
}

describe('DesktopApp', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    resetStore()
  })

  it('renders sidebar and graph canvas when no node selected', async () => {
    render(<DesktopApp />)
    expect(screen.getByTestId('sidebar')).toBeInTheDocument()
    await waitFor(() => {
      expect(screen.getByTestId('graph-canvas')).toBeInTheDocument()
    })
  })

  it('does not render graph canvas when a node is selected', async () => {
    useStore.setState({ selectedNodeId: 'n-1' })
    render(<DesktopApp />)
    await waitFor(() => {
      expect(screen.queryByTestId('graph-canvas')).not.toBeInTheDocument()
    })
  })

  it('renders all panels', async () => {
    render(<DesktopApp />)
    expect(screen.getByTestId('search-panel')).toBeInTheDocument()
    await waitFor(() => {
      expect(screen.getByTestId('chat-panel')).toBeInTheDocument()
      expect(screen.getByTestId('extraction-review')).toBeInTheDocument()
      expect(screen.getByTestId('command-palette')).toBeInTheDocument()
    })
  })

  it('does not render NoteEditor when no node selected', () => {
    render(<DesktopApp />)
    expect(screen.queryByTestId('note-editor')).not.toBeInTheDocument()
  })

  it('renders NoteEditor when node is selected', () => {
    useStore.setState({ selectedNodeId: 'n-1' })
    render(<DesktopApp />)
    expect(screen.getByTestId('note-editor')).toBeInTheDocument()
  })

  it('shows TabBar when openTabs has items', () => {
    useStore.setState({
      selectedNodeId: 'n-1',
      openTabs: [{ id: 'n-1', title: 'Test Note' }],
    })
    render(<DesktopApp />)
    expect(screen.getByTestId('tab-bar')).toBeInTheDocument()
  })

  it('does not render secondary editor when split is off', () => {
    useStore.setState({
      selectedNodeId: 'n-1',
      splitMode: 'off',
      secondaryTabId: 'n-2',
      secondaryNode: { id: 'n-2', vault_id: 'v-1', title: 'Secondary', content: '', content_type: 'note', file_path: null, metadata: null, word_count: 0, created_at: null, updated_at: null },
    })
    render(<DesktopApp />)
    expect(screen.queryByTestId('note-editor-secondary')).not.toBeInTheDocument()
  })

  it('shows split pane placeholder when split is on but no secondary tab', () => {
    useStore.setState({
      selectedNodeId: 'n-1',
      splitMode: 'vertical',
      secondaryTabId: null,
      secondaryNode: null,
    })
    render(<DesktopApp />)
    expect(screen.getByText(/Ctrl\+click a note/)).toBeInTheDocument()
  })

  it('renders secondary editor in vertical split mode', () => {
    useStore.setState({
      selectedNodeId: 'n-1',
      splitMode: 'vertical',
      secondaryTabId: 'n-2',
      secondaryNode: { id: 'n-2', vault_id: 'v-1', title: 'Secondary', content: '', content_type: 'note', file_path: null, metadata: null, word_count: 0, created_at: null, updated_at: null },
    })
    render(<DesktopApp />)
    expect(screen.getByTestId('note-editor-secondary')).toBeInTheDocument()
  })

  describe('keyboard shortcuts', () => {
    it('Ctrl+S calls saveNode', () => {
      const saveNode = vi.fn()
      const orig = useStore.getState().saveNode
      useStore.setState({ saveNode } as any)

      render(<DesktopApp />)
      fireEvent.keyDown(window, { key: 's', ctrlKey: true })

      expect(saveNode).toHaveBeenCalled()
      useStore.setState({ saveNode: orig } as any)
    })

    it('Ctrl+N calls createNode with Untitled', () => {
      const createNode = vi.fn()
      useStore.setState({ createNode } as any)

      render(<DesktopApp />)
      fireEvent.keyDown(window, { key: 'n', ctrlKey: true })

      expect(createNode).toHaveBeenCalledWith('Untitled')
    })

    it('Ctrl+K calls toggleSearch', () => {
      let toggled = false
      useStore.setState({ toggleSearch: () => { toggled = true } } as any)

      render(<DesktopApp />)
      fireEvent.keyDown(window, { key: 'k', ctrlKey: true })

      expect(toggled).toBe(true)
    })

    it('Ctrl+P calls toggleCommandPalette', () => {
      let toggled = false
      useStore.setState({ toggleCommandPalette: () => { toggled = true } } as any)

      render(<DesktopApp />)
      fireEvent.keyDown(window, { key: 'p', ctrlKey: true })

      expect(toggled).toBe(true)
    })

    it('Ctrl+L calls toggleChat', () => {
      let toggled = false
      useStore.setState({ toggleChat: () => { toggled = true } } as any)

      render(<DesktopApp />)
      fireEvent.keyDown(window, { key: 'l', ctrlKey: true })

      expect(toggled).toBe(true)
    })

    it('Ctrl+Shift+E calls toggleExtractions', () => {
      let toggled = false
      useStore.setState({ toggleExtractions: () => { toggled = true } } as any)

      render(<DesktopApp />)
      fireEvent.keyDown(window, { key: 'E', ctrlKey: true, shiftKey: true })

      expect(toggled).toBe(true)
    })

    it('Ctrl+Shift+B calls toggleSidebar', () => {
      let toggled = false
      useStore.setState({ toggleSidebar: () => { toggled = true } } as any)

      render(<DesktopApp />)
      fireEvent.keyDown(window, { key: 'B', ctrlKey: true, shiftKey: true })

      expect(toggled).toBe(true)
    })

    it('Ctrl+O calls toggleCommandPalette', () => {
      let toggled = false
      useStore.setState({ toggleCommandPalette: () => { toggled = true } } as any)

      render(<DesktopApp />)
      fireEvent.keyDown(window, { key: 'o', ctrlKey: true })

      expect(toggled).toBe(true)
    })

    it('Meta+S calls saveNode (macOS)', () => {
      const saveNode = vi.fn()
      useStore.setState({ saveNode } as any)

      render(<DesktopApp />)
      fireEvent.keyDown(window, { key: 's', metaKey: true })

      expect(saveNode).toHaveBeenCalled()
    })
  })

  describe('user CSS', () => {
    it('injects user CSS style element', () => {
      useStore.setState({ userCss: 'body { color: red; }' })
      render(<DesktopApp />)

      const style = document.getElementById('runestone-user-css')
      expect(style).toBeInTheDocument()
      expect(style?.textContent).toBe('body { color: red; }')
    })

    it('removes CSS when userCss is cleared', () => {
      useStore.setState({ userCss: 'body { color: red; }' })
      const { rerender } = render(<DesktopApp />)

      useStore.setState({ userCss: '' })
      rerender(<DesktopApp />)

      const style = document.getElementById('runestone-user-css')
      expect(style?.textContent).toBe('')
    })
  })
})
