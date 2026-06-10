import { describe, it, expect, beforeEach } from 'vitest'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { ChatPanel } from '@/features/chat'
import { useStore } from '@/store'

function resetStore() {
  useStore.setState({
    showChat: false,
    chatMessages: [],
    chatLoading: false,
    chatCitations: [],
    chatAnswer: null,
    selectedVaultId: null,
    vaults: [],
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
    tagSuggestions: [],
    importResult: null,
  })
}

describe('ChatPanel', () => {
  beforeEach(() => {
    resetStore()
  })

  it('renders nothing when closed', () => {
    const { container } = render(<ChatPanel />)
    expect(container.innerHTML).toBe('')
  })

  it('renders chat interface when open', () => {
    useStore.setState({ showChat: true })
    render(<ChatPanel />)
    expect(screen.getByText('Chat with Graph')).toBeInTheDocument()
    expect(screen.getByPlaceholderText('Ask about your knowledge graph...')).toBeInTheDocument()
    expect(screen.getByText('Send')).toBeInTheDocument()
  })

  it('closes on close button click', async () => {
    const user = userEvent.setup()
    useStore.setState({ showChat: true })
    render(<ChatPanel />)
    await user.click(screen.getByText('×'))
    expect(useStore.getState().showChat).toBe(false)
  })

  it('sends message on Send button click', async () => {
    const user = userEvent.setup()
    useStore.setState({ showChat: true, selectedVaultId: 'v-1' })
    render(<ChatPanel />)

    const input = screen.getByPlaceholderText('Ask about your knowledge graph...')
    await user.type(input, 'Hello')
    await user.click(screen.getByText('Send'))

    expect(useStore.getState().chatMessages).toHaveLength(2)
    expect(useStore.getState().chatMessages[0].role).toBe('user')
    expect(useStore.getState().chatMessages[0].content).toBe('Hello')
    expect(useStore.getState().chatMessages[1].role).toBe('assistant')
  })

  it('sends message on Enter', async () => {
    const user = userEvent.setup()
    useStore.setState({ showChat: true, selectedVaultId: 'v-1' })
    render(<ChatPanel />)

    const input = screen.getByPlaceholderText('Ask about your knowledge graph...')
    await user.type(input, 'Question{Enter}')

    expect(useStore.getState().chatMessages).toHaveLength(2)
  })

  it('renders chat messages', () => {
    useStore.setState({
      showChat: true,
      chatMessages: [
        { role: 'user', content: 'Hello' },
        { role: 'assistant', content: 'Hi there!' },
      ],
    })
    render(<ChatPanel />)
    expect(screen.getByText('Hello')).toBeInTheDocument()
    expect(screen.getByText('Hi there!')).toBeInTheDocument()
  })

  it('renders citations', () => {
    useStore.setState({
      showChat: true,
      chatCitations: [
        { node_id: 'n-1', title: 'Source 1', snippet: 'This is a source snippet' },
      ],
    })
    render(<ChatPanel />)
    expect(screen.getByText('Sources')).toBeInTheDocument()
    expect(screen.getByText('Source 1')).toBeInTheDocument()
  })

  it('shows loading indicator', () => {
    useStore.setState({
      showChat: true,
      chatLoading: true,
      chatMessages: [{ role: 'assistant', content: '' }],
    })
    render(<ChatPanel />)
    expect(screen.getByText('Thinking...')).toBeInTheDocument()
  })

  it('disables Send button when loading', () => {
    useStore.setState({ showChat: true, chatLoading: true })
    render(<ChatPanel />)
    expect(screen.getByText('Send')).toBeDisabled()
  })
})
