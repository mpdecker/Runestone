import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import { PropertiesPanel } from '@/features/sidebar/PropertiesPanel'
import { useStore } from '@/store'

function resetStore() {
  useStore.setState({
    vaults: [],
    selectedVaultId: 'v-1',
    nodes: [],
    selectedNodeId: null,
    currentNode: null,
    nodeProperties: [],
    openTabs: [],
    graphData: null,
    error: null,
    isLoading: false,
  })
}

describe('PropertiesPanel', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    resetStore()
  })

  it('returns null when no node selected', () => {
    const { container } = render(<PropertiesPanel />)

    expect(container.innerHTML).toBe('')
  })

  it('renders properties list', () => {
    useStore.setState({
      selectedNodeId: 'n-1',
      nodeProperties: [
        { key: 'priority', value: 'high', prop_type: 'text' },
        { key: 'count', value: 42, prop_type: 'number' },
      ],
    })
    render(<PropertiesPanel />)

    expect(screen.getByText('priority:')).toBeInTheDocument()
    expect(screen.getByText('high')).toBeInTheDocument()
  })

  it('shows empty state when no properties', () => {
    useStore.setState({ selectedNodeId: 'n-1', nodeProperties: [] })
    render(<PropertiesPanel />)

    expect(screen.getByText('No custom properties')).toBeInTheDocument()
  })

  it('renders add property form with type selector', () => {
    useStore.setState({ selectedNodeId: 'n-1' })
    render(<PropertiesPanel />)

    expect(screen.getByPlaceholderText('Key')).toBeInTheDocument()
    expect(screen.getByDisplayValue('Text')).toBeInTheDocument()
    expect(screen.getByText('Add')).toBeInTheDocument()
  })

  it('changes value placeholder based on type selection', () => {
    useStore.setState({ selectedNodeId: 'n-1' })
    render(<PropertiesPanel />)

    fireEvent.change(screen.getByRole('combobox'), { target: { value: 'number' } })
    expect(screen.getByPlaceholderText('Number')).toBeInTheDocument()

    fireEvent.change(screen.getByRole('combobox'), { target: { value: 'checkbox' } })
    expect(screen.getByPlaceholderText('true/false')).toBeInTheDocument()
  })

  it('calls setProperty on add with correct type coercion', async () => {
    const setProperty = vi.fn().mockResolvedValue(undefined)
    useStore.setState({ selectedNodeId: 'n-1', setProperty } as any)
    render(<PropertiesPanel />)

    fireEvent.change(screen.getByPlaceholderText('Key'), { target: { value: 'count' } })
    fireEvent.change(screen.getByRole('combobox'), { target: { value: 'number' } })
    fireEvent.change(screen.getByPlaceholderText('Number'), { target: { value: '42' } })
    fireEvent.click(screen.getByText('Add'))

    expect(setProperty).toHaveBeenCalledWith('n-1', 'count', 42)
  })

  it('calls setProperty with boolean for checkbox type', async () => {
    const setProperty = vi.fn().mockResolvedValue(undefined)
    useStore.setState({ selectedNodeId: 'n-1', setProperty } as any)
    render(<PropertiesPanel />)

    fireEvent.change(screen.getByPlaceholderText('Key'), { target: { value: 'enabled' } })
    fireEvent.change(screen.getByRole('combobox'), { target: { value: 'checkbox' } })
    fireEvent.change(screen.getByPlaceholderText('true/false'), { target: { value: 'true' } })
    fireEvent.click(screen.getByText('Add'))

    expect(setProperty).toHaveBeenCalledWith('n-1', 'enabled', true)
  })

  it('does not add property when key is empty', () => {
    const setProperty = vi.fn()
    useStore.setState({ selectedNodeId: 'n-1', setProperty } as any)
    render(<PropertiesPanel />)

    fireEvent.click(screen.getByText('Add'))

    expect(setProperty).not.toHaveBeenCalled()
  })

  it('supports editing a property on double-click', () => {
    useStore.setState({
      selectedNodeId: 'n-1',
      nodeProperties: [{ key: 'status', value: 'draft', prop_type: 'text' }],
    })
    render(<PropertiesPanel />)

    fireEvent.click(screen.getByText('draft'))

    expect(screen.getByDisplayValue('draft')).toBeInTheDocument()
  })

  it('saves edit on Enter key', async () => {
    const setProperty = vi.fn().mockResolvedValue(undefined)
    useStore.setState({
      selectedNodeId: 'n-1',
      setProperty,
      nodeProperties: [{ key: 'status', value: 'old', prop_type: 'text' }],
    } as any)
    render(<PropertiesPanel />)

    fireEvent.click(screen.getByText('old'))
    const input = screen.getByDisplayValue('old')
    fireEvent.change(input, { target: { value: 'new' } })
    fireEvent.keyDown(input, { key: 'Enter' })

    expect(setProperty).toHaveBeenCalledWith('n-1', 'status', 'new')
  })

  it('cancels edit on Escape key', () => {
    useStore.setState({
      selectedNodeId: 'n-1',
      nodeProperties: [{ key: 'status', value: 'draft', prop_type: 'text' }],
    })
    render(<PropertiesPanel />)

    fireEvent.click(screen.getByText('draft'))
    const input = screen.getByDisplayValue('draft')
    fireEvent.keyDown(input, { key: 'Escape' })

    expect(screen.queryByDisplayValue('draft')).not.toBeInTheDocument()
  })
})
