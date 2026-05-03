import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import { TabBar } from '@/features/editor/TabBar'
import { useStore } from '@/store'

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
    nodeProperties: [],
    nodeTags: null,
    graphData: null,
    error: null,
    isLoading: false,
  })
}

describe('TabBar', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    resetStore()
  })

  it('returns null when no tabs are open', () => {
    const { container } = render(<TabBar />)
    expect(container.innerHTML).toBe('')
  })

  it('renders tab items', () => {
    useStore.setState({
      openTabs: [
        { id: 'n-1', title: 'Note One' },
        { id: 'n-2', title: 'Note Two' },
      ],
    })
    render(<TabBar />)

    expect(screen.getByText('Note One')).toBeInTheDocument()
    expect(screen.getByText('Note Two')).toBeInTheDocument()
  })

  it('highlights active tab', () => {
    useStore.setState({
      openTabs: [
        { id: 'n-1', title: 'Active' },
        { id: 'n-2', title: 'Inactive' },
      ],
      activeTabId: 'n-1',
    })
    render(<TabBar />)

    const active = screen.getByText('Active').closest('.group')
    const inactive = screen.getByText('Inactive').closest('.group')

    expect(active?.className).toContain('bg-background')
    expect(inactive?.className).not.toContain('bg-background')
  })

  it('calls switchToTab on tab click', () => {
    let switched: string | null = null
    useStore.setState({
      openTabs: [{ id: 'n-1', title: 'Note' }],
      switchToTab: (id: string) => { switched = id },
    } as any)
    render(<TabBar />)

    fireEvent.click(screen.getByText('Note'))

    expect(switched).toBe('n-1')
  })

  it('calls closeTab on close button click with stopPropagation', () => {
    let closed: string | null = null
    let switched: string | null = null
    useStore.setState({
      openTabs: [{ id: 'n-1', title: 'Note' }],
      closeTab: (id: string) => { closed = id },
      switchToTab: (id: string) => { switched = id },
    } as any)
    render(<TabBar />)

    const closeBtn = screen.getByTitle('Close tab')
    fireEvent.click(closeBtn)

    expect(closed).toBe('n-1')
    expect(switched).toBeNull()
  })
})
