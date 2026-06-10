import { describe, it, expect, beforeEach } from 'vitest'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { Sidebar } from '@/features/sidebar'
import { useStore } from '@/store'
import { makeNodeListItem } from '@/__tests__/helpers/fixtures'

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

const sampleVaults = [
  { id: 'v-1', name: 'Vault One', root_path: '/v1', created_at: null, updated_at: null },
  { id: 'v-2', name: 'Vault Two', root_path: '/v2', created_at: null, updated_at: null },
]

const sampleNodes = [
  makeNodeListItem({ id: 'n-1', title: 'Note One', content_type: 'note' }),
  makeNodeListItem({ id: 'n-2', title: 'Note Two', content_type: 'concept' }),
]

describe('Sidebar', () => {
  beforeEach(() => {
    resetStore()
  })

  it('renders vaults header', () => {
    render(<Sidebar />)
    expect(screen.getByText('Vaults')).toBeInTheDocument()
  })

  it('shows empty vault state', () => {
    render(<Sidebar />)
    expect(screen.getByText('No vaults yet')).toBeInTheDocument()
  })

  it('renders vault list', () => {
    useStore.setState({ vaults: sampleVaults })
    render(<Sidebar />)
    expect(screen.getByText('Vault One')).toBeInTheDocument()
    expect(screen.getByText('Vault Two')).toBeInTheDocument()
  })

  it('shows new vault form on button click', async () => {
    const user = userEvent.setup()
    render(<Sidebar />)
    const addButtons = screen.getAllByText('+')
    await user.click(addButtons[0])
    expect(screen.getByPlaceholderText('Vault name')).toBeInTheDocument()
    expect(screen.getByPlaceholderText('Root path')).toBeInTheDocument()
  })

  it('shows notes section when vault selected', () => {
    useStore.setState({ selectedVaultId: 'v-1', nodes: sampleNodes })
    render(<Sidebar />)
    expect(screen.getByText('Notes')).toBeInTheDocument()
    expect(screen.getByText('Note One')).toBeInTheDocument()
    expect(screen.getByText('Note Two')).toBeInTheDocument()
  })

  it('showSearch button toggles search', async () => {
    const user = userEvent.setup()
    useStore.setState({ selectedVaultId: 'v-1' })
    render(<Sidebar />)
    await user.click(screen.getByText('Q'))
    expect(useStore.getState().showSearch).toBe(true)
  })

  it('showChat button toggles chat', async () => {
    const user = userEvent.setup()
    useStore.setState({ selectedVaultId: 'v-1' })
    render(<Sidebar />)
    await user.click(screen.getByText('C'))
    expect(useStore.getState().showChat).toBe(true)
  })

  it('shows pending extractions count', () => {
    useStore.setState({
      selectedVaultId: 'v-1',
      pendingExtractions: [
        { id: 'e-1', title: 'E1', content_type: 'entity', metadata: null, created_at: null },
        { id: 'e-2', title: 'E2', content_type: 'entity', metadata: null, created_at: null },
      ],
    })
    render(<Sidebar />)
    // The "E" button has a badge with the count
    expect(screen.getByText('2')).toBeInTheDocument()
  })

  it('shows filter text input', () => {
    useStore.setState({ selectedVaultId: 'v-1' })
    render(<Sidebar />)
    expect(screen.getByPlaceholderText('Search notes...')).toBeInTheDocument()
  })

  it('updates filter text on input', async () => {
    const user = userEvent.setup()
    useStore.setState({ selectedVaultId: 'v-1' })
    render(<Sidebar />)
    const input = screen.getByPlaceholderText('Search notes...')
    await user.type(input, 'test')
    expect(useStore.getState().filterText).toBe('test')
  })

  it('shows node type filter buttons', () => {
    useStore.setState({ selectedVaultId: 'v-1' })
    render(<Sidebar />)
    expect(screen.getByText('note')).toBeInTheDocument()
    expect(screen.getByText('concept')).toBeInTheDocument()
    expect(screen.getByText('entity')).toBeInTheDocument()
    expect(screen.getByText('document')).toBeInTheDocument()
  })

  it('toggles filter type on click', async () => {
    const user = userEvent.setup()
    useStore.setState({ selectedVaultId: 'v-1' })
    render(<Sidebar />)
    await user.click(screen.getByText('note'))
    expect(useStore.getState().filterTypes).toContain('note')
    await user.click(screen.getByText('note'))
    expect(useStore.getState().filterTypes).not.toContain('note')
  })

  it('shows error banner', () => {
    useStore.setState({ error: 'Something went wrong' })
    render(<Sidebar />)
    expect(screen.getByText('Something went wrong')).toBeInTheDocument()
  })

  it('shows loading indicator', () => {
    useStore.setState({ isLoading: true })
    render(<Sidebar />)
    expect(screen.getByText('Loading...')).toBeInTheDocument()
  })

  it('filters nodes by filter text', () => {
    useStore.setState({
      selectedVaultId: 'v-1',
      nodes: sampleNodes,
      filterText: 'Note One',
    })
    render(<Sidebar />)
    expect(screen.getByText('Note One')).toBeInTheDocument()
    expect(screen.queryByText('Note Two')).not.toBeInTheDocument()
  })

  it('shows backlinks section when selected node has backlinks', () => {
    useStore.setState({
      selectedVaultId: 'v-1',
      selectedNodeId: 'n-1',
      showBacklinks: true,
      backlinks: [{ node_id: 'n-2', title: 'Note Two', content_type: 'note' }],
    })
    render(<Sidebar />)
    expect(screen.getByText('Backlinks')).toBeInTheDocument()
    expect(screen.getAllByText('Note Two').length).toBeGreaterThanOrEqual(1)
  })

  it('shows no backlinks message', () => {
    useStore.setState({
      selectedVaultId: 'v-1',
      selectedNodeId: 'n-1',
      showBacklinks: true,
      backlinks: [],
    })
    render(<Sidebar />)
    expect(screen.getByText('No backlinks found')).toBeInTheDocument()
  })

  it('shows suggested links', () => {
    useStore.setState({
      selectedVaultId: 'v-1',
      selectedNodeId: 'n-1',
      suggestedLinks: [{ node_id: 'n-3', title: 'Suggested', content_type: 'note', snippet: '', score: 0.9 }],
    })
    render(<Sidebar />)
    expect(screen.getByText('Suggested Links')).toBeInTheDocument()
    expect(screen.getByText('Suggested')).toBeInTheDocument()
  })

  it('shows tag suggestions', () => {
    useStore.setState({
      selectedVaultId: 'v-1',
      selectedNodeId: 'n-1',
      tagSuggestions: [{ name: 'ai', confidence: 0.95, reason: 'relevant' }],
    })
    render(<Sidebar />)
    expect(screen.getByText('Suggested Tags')).toBeInTheDocument()
    expect(screen.getByText('#ai (95%)')).toBeInTheDocument()
  })

  it('shows node summary', () => {
    useStore.setState({
      selectedVaultId: 'v-1',
      selectedNodeId: 'n-1',
      nodeSummary: 'This is a summary of the note.',
    })
    render(<Sidebar />)
    expect(screen.getByText('This is a summary of the note.')).toBeInTheDocument()
  })

  it('shows Summarize button as disabled when loading', () => {
    useStore.setState({
      selectedVaultId: 'v-1',
      selectedNodeId: 'n-1',
      summaryLoading: true,
    })
    render(<Sidebar />)
    expect(screen.getByText('...')).toBeInTheDocument()
  })

  it('shows new note form on + click', async () => {
    const user = userEvent.setup()
    useStore.setState({ selectedVaultId: 'v-1' })
    render(<Sidebar />)

    const plusButtons = screen.getAllByText('+')
    await user.click(plusButtons[1])
    expect(screen.getByPlaceholderText('Note title')).toBeInTheDocument()
  })
})
