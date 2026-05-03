import { describe, it, expect, beforeEach } from 'vitest'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { CommandPalette } from '@/features/command-palette'
import { useStore } from '@/store'

function resetStore() {
  useStore.setState({
    showCommandPalette: false,
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
    showChat: false,
    chatMessages: [],
    chatLoading: false,
    chatCitations: [],
    chatAnswer: null,
    tagSuggestions: [],
    importResult: null,
  })
}

describe('CommandPalette', () => {
  beforeEach(() => {
    resetStore()
  })

  it('renders nothing when closed', () => {
    const { container } = render(<CommandPalette />)
    expect(container.innerHTML).toBe('')
  })

  it('renders commands when open', () => {
    useStore.setState({ showCommandPalette: true })
    render(<CommandPalette />)
    expect(screen.getByPlaceholderText('Search notes or commands...')).toBeInTheDocument()
    expect(screen.getByText('New Note')).toBeInTheDocument()
    expect(screen.getByText('Search')).toBeInTheDocument()
    expect(screen.getByText('Extractions')).toBeInTheDocument()
  })

  it('shows keyboard shortcuts', () => {
    useStore.setState({ showCommandPalette: true })
    render(<CommandPalette />)
    expect(screen.getByText('Ctrl+N')).toBeInTheDocument()
    expect(screen.getByText('Ctrl+K')).toBeInTheDocument()
    expect(screen.getByText('Ctrl+Shift+E')).toBeInTheDocument()
  })

  it('closes on Escape', async () => {
    const user = userEvent.setup()
    useStore.setState({ showCommandPalette: true })
    render(<CommandPalette />)

    const input = screen.getByPlaceholderText('Search notes or commands...')
    await user.click(input)
    await user.keyboard('{Escape}')
    expect(useStore.getState().showCommandPalette).toBe(false)
  })

  it('closes when clicking backdrop', async () => {
    const user = userEvent.setup()
    useStore.setState({ showCommandPalette: true })
    render(<CommandPalette />)

    const backdrop = document.querySelector('.fixed.inset-0') as HTMLElement
    await user.click(backdrop)
    expect(useStore.getState().showCommandPalette).toBe(false)
  })

  it('executes New Note command', async () => {
    const user = userEvent.setup()
    useStore.setState({ showCommandPalette: true })
    render(<CommandPalette />)

    await user.click(screen.getByText('New Note'))
    expect(useStore.getState().showCommandPalette).toBe(false)
  })

  it('executes Search command', async () => {
    const user = userEvent.setup()
    useStore.setState({ showCommandPalette: true })
    render(<CommandPalette />)

    await user.click(screen.getByText('Search'))
    expect(useStore.getState().showCommandPalette).toBe(false)
    expect(useStore.getState().showSearch).toBe(true)
  })

  it('executes Extractions command', async () => {
    const user = userEvent.setup()
    useStore.setState({ showCommandPalette: true })
    render(<CommandPalette />)

    await user.click(screen.getByText('Extractions'))
    expect(useStore.getState().showCommandPalette).toBe(false)
    expect(useStore.getState().showExtractions).toBe(true)
  })
})
