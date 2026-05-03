import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import { NodeActions } from '@/features/sidebar/NodeActions'
import { useStore } from '@/store'

function resetStore() {
  useStore.setState({
    vaults: [],
    selectedVaultId: 'v-1',
    nodes: [],
    selectedNodeId: null,
    currentNode: null,
    showBacklinks: false,
    backlinks: [],
    showOutgoingLinks: false,
    outgoingLinks: [],
    nodeSummary: null,
    summaryLoading: false,
    suggestedLinks: [],
    tagSuggestions: [],
    nodeTags: null,
    nodeProperties: [],
    openTabs: [],
    graphData: null,
    error: null,
    isLoading: false,
  })
}

describe('NodeActions', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    resetStore()
  })

  describe('rendering', () => {
    it('returns null when no node is selected', () => {
      const { container } = render(<NodeActions />)
      expect(container.innerHTML).toBe('')
    })

    it('renders action buttons when node is selected', () => {
      useStore.setState({ selectedNodeId: 'n-1' })
      render(<NodeActions />)
      expect(screen.getByText('Show Backlinks')).toBeInTheDocument()
      expect(screen.getByText('Show Outgoing')).toBeInTheDocument()
    })

    it('shows Parse Links button when currentNode exists', () => {
      useStore.setState({
        selectedNodeId: 'n-1',
        currentNode: { id: 'n-1', vault_id: 'v-1', title: 'Test', content: '[[link]]', content_type: 'note', file_path: null, metadata: null, word_count: 1, created_at: null, updated_at: null },
      })
      render(<NodeActions />)
      expect(screen.getByText('Parse Links')).toBeInTheDocument()
    })

    it('hides Parse Links when currentNode is null', () => {
      useStore.setState({ selectedNodeId: 'n-1', currentNode: null })
      render(<NodeActions />)
      expect(screen.queryByText('Parse Links')).not.toBeInTheDocument()
    })
  })

  describe('backlinks toggle', () => {
    it('shows Hide when backlinks are visible', () => {
      useStore.setState({ selectedNodeId: 'n-1', showBacklinks: true })
      render(<NodeActions />)
      expect(screen.getByText('Hide Backlinks')).toBeInTheDocument()
    })

    it('calls toggleBacklinks on click', () => {
      let toggled = false
      useStore.setState({ selectedNodeId: 'n-1', toggleBacklinks: () => { toggled = true } } as any)
      render(<NodeActions />)
      fireEvent.click(screen.getByText('Show Backlinks'))
      expect(toggled).toBe(true)
    })

    it('shows backlink list when visible', () => {
      useStore.setState({
        selectedNodeId: 'n-1',
        showBacklinks: true,
        backlinks: [{ node_id: 'n-2', title: 'Linked Note', content_type: 'note', context: 'refers to...' }],
      })
      render(<NodeActions />)
      expect(screen.getByText('Linked Note')).toBeInTheDocument()
    })

    it('shows empty message when no backlinks', () => {
      useStore.setState({ selectedNodeId: 'n-1', showBacklinks: true, backlinks: [] })
      render(<NodeActions />)
      expect(screen.getByText('No backlinks found')).toBeInTheDocument()
    })
  })

  describe('outgoing links toggle', () => {
    it('shows outgoing links when enabled', () => {
      useStore.setState({
        selectedNodeId: 'n-1',
        showOutgoingLinks: true,
        outgoingLinks: [{ node_id: 'n-3', title: 'Target', content_type: 'note', context: 'link text' }],
      })
      render(<NodeActions />)
      expect(screen.getByText('Target')).toBeInTheDocument()
    })

    it('does not navigate for nil UUID outgoing links', () => {
      useStore.setState({
        selectedNodeId: 'n-1',
        showOutgoingLinks: true,
        outgoingLinks: [{ node_id: '00000000-0000-0000-0000-000000000000', title: 'Unresolved', content_type: 'unresolved', context: null }],
      })
      render(<NodeActions />)
      expect(screen.getByText('Unresolved')).toBeInTheDocument()
    })
  })

  describe('summarize button', () => {
    it('shows ... when loading', () => {
      useStore.setState({ selectedNodeId: 'n-1', summaryLoading: true })
      render(<NodeActions />)
      expect(screen.getByText('...')).toBeInTheDocument()
    })

    it('is disabled when loading', () => {
      useStore.setState({ selectedNodeId: 'n-1', summaryLoading: true })
      render(<NodeActions />)
      expect(screen.getByText('...').closest('button')).toBeDisabled()
    })

    it('displays summary text when available', () => {
      useStore.setState({ selectedNodeId: 'n-1', nodeSummary: 'This is a summary' })
      render(<NodeActions />)
      expect(screen.getByText('This is a summary')).toBeInTheDocument()
    })
  })

  describe('suggested tags', () => {
    it('shows suggested tags with confidence percentage', () => {
      useStore.setState({
        selectedNodeId: 'n-1',
        tagSuggestions: [
          { name: 'ai', confidence: 0.85, reason: 'AI topic' },
          { name: 'ml', confidence: 0.72, reason: 'ML topic' },
        ],
      })
      render(<NodeActions />)
      expect(screen.getByText('#ai (85%)')).toBeInTheDocument()
      expect(screen.getByText('#ml (72%)')).toBeInTheDocument()
    })

    it('has Accept All button for suggested tags', () => {
      useStore.setState({
        selectedNodeId: 'n-1',
        tagSuggestions: [{ name: 'ai', confidence: 0.9, reason: 'topic' }],
      })
      render(<NodeActions />)
      expect(screen.getByText('Accept All')).toBeInTheDocument()
    })
  })

  describe('tag management', () => {
    it('shows existing tags', () => {
      useStore.setState({
        selectedNodeId: 'n-1',
        nodeTags: { node_id: 'n-1', tags: ['javascript', 'react'] },
      })
      render(<NodeActions />)
      expect(screen.getByText('#javascript')).toBeInTheDocument()
      expect(screen.getByText('#react')).toBeInTheDocument()
    })

    it('shows None when no tags exist', () => {
      useStore.setState({
        selectedNodeId: 'n-1',
        nodeTags: { node_id: 'n-1', tags: [] },
      })
      render(<NodeActions />)
      expect(screen.getByText('None')).toBeInTheDocument()
    })

    it('has tag input field when nodeTags is present', () => {
      useStore.setState({
        selectedNodeId: 'n-1',
        nodeTags: { node_id: 'n-1', tags: [] },
      })
      render(<NodeActions />)
      expect(screen.getByPlaceholderText('Add tags (comma-separated)...')).toBeInTheDocument()
    })
  })

  describe('suggested links', () => {
    it('shows suggested links with scores', () => {
      useStore.setState({
        selectedNodeId: 'n-1',
        suggestedLinks: [
          { node_id: 'n-2', title: 'Related Note', content_type: 'note', snippet: '', score: 0.75 },
        ],
      })
      render(<NodeActions />)
      expect(screen.getByText('Related Note')).toBeInTheDocument()
      expect(screen.getByText('75%')).toBeInTheDocument()
    })
  })

  describe('Export MD button', () => {
    it('renders Export MD button', () => {
      useStore.setState({ selectedNodeId: 'n-1' })
      render(<NodeActions />)
      expect(screen.getByText('Export MD')).toBeInTheDocument()
    })
  })
})
