import { describe, it, expect, beforeEach } from 'vitest'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { SearchPanel } from '@/features/search'
import { useStore } from '@/store'

function resetStore() {
  useStore.setState({
    showSearch: false,
    searchQuery: '',
    searchResults: null,
    searchLoading: false,
    similarNodes: [],
    selectedNodeId: null,
    vaults: [],
    selectedVaultId: null,
    nodes: [],
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

describe('SearchPanel', () => {
  beforeEach(() => {
    resetStore()
  })

  it('renders nothing when closed', () => {
    const { container } = render(<SearchPanel />)
    expect(container.innerHTML).toBe('')
  })

  it('renders search input when open', () => {
    useStore.setState({ showSearch: true })
    render(<SearchPanel />)
    expect(screen.getByPlaceholderText('Search notes semantically...')).toBeInTheDocument()
    expect(screen.getByText('Go')).toBeInTheDocument()
  })

  it('shows close button', () => {
    useStore.setState({ showSearch: true })
    render(<SearchPanel />)
    expect(screen.getByText('×')).toBeInTheDocument()
  })

  it('closes on close button click', async () => {
    const user = userEvent.setup()
    useStore.setState({ showSearch: true })
    render(<SearchPanel />)
    await user.click(screen.getByText('×'))
    expect(useStore.getState().showSearch).toBe(false)
  })

  it('shows "Find similar" link when node selected', () => {
    useStore.setState({ showSearch: true, selectedNodeId: 'n-1' })
    render(<SearchPanel />)
    expect(screen.getByText('Find similar to current note')).toBeInTheDocument()
  })

  it('hides "Find similar" link when no node selected', () => {
    useStore.setState({ showSearch: true, selectedNodeId: null })
    render(<SearchPanel />)
    expect(screen.queryByText('Find similar to current note')).not.toBeInTheDocument()
  })

  it('updates search query on input', async () => {
    const user = userEvent.setup()
    useStore.setState({ showSearch: true })
    render(<SearchPanel />)
    const input = screen.getByPlaceholderText('Search notes semantically...')
    await user.type(input, 'test')
    expect(useStore.getState().searchQuery).toBe('test')
  })

  it('runs search on Enter', async () => {
    const user = userEvent.setup()
    useStore.setState({ showSearch: true, selectedVaultId: 'v-1' })
    render(<SearchPanel />)
    const input = screen.getByPlaceholderText('Search notes semantically...')
    await user.type(input, 'query')
    await user.keyboard('{Enter}')
    expect(useStore.getState().searchQuery).toBe('query')
  })

  it('runs search on Go button click', async () => {
    const user = userEvent.setup()
    useStore.setState({ showSearch: true, selectedVaultId: 'v-1', searchQuery: 'hello' })
    render(<SearchPanel />)
    await user.click(screen.getByText('Go'))
  })

  it('shows searching indicator', () => {
    useStore.setState({ showSearch: true, searchLoading: true })
    render(<SearchPanel />)
    expect(screen.getByText('Searching...')).toBeInTheDocument()
  })

  it('renders search results with type and score', () => {
    useStore.setState({
      showSearch: true,
      searchResults: {
        vector_results: [],
        fts_results: [],
        combined: [
          { node_id: 'n-1', title: 'Note 1', content_type: 'note', snippet: 'Snippet 1', score: 0.95 },
          { node_id: 'n-2', title: 'Note 2', content_type: 'concept', snippet: 'Snippet 2', score: 0.80 },
        ],
      },
    })
    render(<SearchPanel />)
    expect(screen.getByText('Combined Results (2)')).toBeInTheDocument()
    expect(screen.getByText('Note 1')).toBeInTheDocument()
    expect(screen.getByText('Note 2')).toBeInTheDocument()
    expect(screen.getByText('note')).toBeInTheDocument()
    expect(screen.getByText('concept')).toBeInTheDocument()
  })

  it('shows no results message', () => {
    useStore.setState({
      showSearch: true,
      searchResults: { vector_results: [], fts_results: [], combined: [] },
    })
    render(<SearchPanel />)
    expect(screen.getByText('No results found')).toBeInTheDocument()
  })

  it('renders similar nodes', () => {
    useStore.setState({
      showSearch: true,
      similarNodes: [
        { node_id: 'n-3', title: 'Similar Note', content_type: 'note', snippet: 's', score: 0.85 },
      ],
    })
    render(<SearchPanel />)
    expect(screen.getByText('Similar Notes (1)')).toBeInTheDocument()
    expect(screen.getByText('Similar Note')).toBeInTheDocument()
  })
})
