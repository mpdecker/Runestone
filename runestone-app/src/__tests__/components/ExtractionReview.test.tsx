import { describe, it, expect, beforeEach } from 'vitest'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { ExtractionReview } from '@/features/extraction'
import { useStore } from '@/store'

function resetStore() {
  useStore.setState({
    showExtractions: false,
    pendingExtractions: [],
    extractionResults: [],
    selectedNodeId: null,
    isLoading: false,
    vaults: [],
    selectedVaultId: null,
    nodes: [],
    currentNode: null,
    isEditorDirty: false,
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

const samplePending = [
  {
    id: 'e-1',
    title: 'Entity One',
    content_type: 'entity',
    metadata: {
      status: 'pending',
      confidence: 0.85,
      source_chunk: 0,
      description: 'A test entity',
      extraction_type: 'entity',
    },
    created_at: null,
  },
  {
    id: 'e-2',
    title: 'Concept One',
    content_type: 'concept',
    metadata: {
      status: 'pending',
      confidence: 0.9,
      source_chunk: 1,
      description: 'A test concept',
      extraction_type: 'concept',
    },
    created_at: null,
  },
]

describe('ExtractionReview', () => {
  beforeEach(() => {
    resetStore()
  })

  it('renders nothing when closed', () => {
    const { container } = render(<ExtractionReview />)
    expect(container.innerHTML).toBe('')
  })

  it('renders extraction panel when open', () => {
    useStore.setState({ showExtractions: true })
    render(<ExtractionReview />)
    expect(screen.getByText('Extractions')).toBeInTheDocument()
  })

  it('shows empty state', () => {
    useStore.setState({ showExtractions: true })
    render(<ExtractionReview />)
    expect(screen.getByText(/No pending extractions/)).toBeInTheDocument()
  })

  it('renders pending extractions', () => {
    useStore.setState({ showExtractions: true, pendingExtractions: samplePending })
    render(<ExtractionReview />)
    expect(screen.getByText('Entity One')).toBeInTheDocument()
    expect(screen.getByText('Concept One')).toBeInTheDocument()
    expect(screen.getByText('entity')).toBeInTheDocument()
    expect(screen.getByText('concept')).toBeInTheDocument()
  })

  it('renders extraction results count', () => {
    useStore.setState({
      showExtractions: true,
      extractionResults: [
        { name: 'E1', description: 'd', extraction_type: 'entity', confidence: 0.8, source_node_id: 'n-1', chunk_index: 0 },
      ],
    })
    render(<ExtractionReview />)
    expect(screen.getByText('Last extraction: 1 items found')).toBeInTheDocument()
  })

  it('shows approve and reject buttons for each extraction', () => {
    useStore.setState({ showExtractions: true, pendingExtractions: [samplePending[0]] })
    render(<ExtractionReview />)
    expect(screen.getByText('Approve')).toBeInTheDocument()
    expect(screen.getByText('Reject')).toBeInTheDocument()
  })

  it('closes on close button click', async () => {
    const user = userEvent.setup()
    useStore.setState({ showExtractions: true })
    render(<ExtractionReview />)
    await user.click(screen.getByText('×'))
    expect(useStore.getState().showExtractions).toBe(false)
  })

  it('checks and unchecks extractions', async () => {
    const user = userEvent.setup()
    useStore.setState({ showExtractions: true, pendingExtractions: samplePending })
    render(<ExtractionReview />)

    const checkboxes = screen.getAllByRole('checkbox')
    await user.click(checkboxes[0])
    await user.click(checkboxes[1])
  })

  it('selects all extractions', async () => {
    const user = userEvent.setup()
    useStore.setState({ showExtractions: true, pendingExtractions: samplePending })
    render(<ExtractionReview />)

    await user.click(screen.getByText('All'))
    const checkboxes = screen.getAllByRole('checkbox')
    expect(checkboxes[0]).toBeChecked()
    expect(checkboxes[1]).toBeChecked()
  })

  it('clears selection', async () => {
    const user = userEvent.setup()
    useStore.setState({ showExtractions: true, pendingExtractions: samplePending })
    render(<ExtractionReview />)

    await user.click(screen.getByText('All'))
    await user.click(screen.getByText('None'))
    const checkboxes = screen.getAllByRole('checkbox')
    expect(checkboxes[0]).not.toBeChecked()
    expect(checkboxes[1]).not.toBeChecked()
  })

  it('shows extract button when node selected', () => {
    useStore.setState({ showExtractions: true, selectedNodeId: 'n-1' })
    render(<ExtractionReview />)
    expect(screen.getByText('Extract from selected document')).toBeInTheDocument()
  })

  it('shows loading state on extract button', () => {
    useStore.setState({ showExtractions: true, selectedNodeId: 'n-1', isLoading: true })
    render(<ExtractionReview />)
    expect(screen.getByText('Extracting...')).toBeInTheDocument()
  })
})
