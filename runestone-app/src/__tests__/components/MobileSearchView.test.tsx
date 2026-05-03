import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import { MobileSearchView } from '@/features/layout/MobileSearchView'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

describe('MobileSearchView', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('renders search input', () => {
    render(<MobileSearchView />)
    expect(screen.getByPlaceholderText('Search your knowledge graph...')).toBeInTheDocument()
  })

  it('shows placeholder when no query', () => {
    render(<MobileSearchView />)
    expect(screen.getByText('Search across your notes')).toBeInTheDocument()
  })

  it('searches on button click', async () => {
    const { invoke } = await import('@tauri-apps/api/core')
    const mockInvoke = invoke as ReturnType<typeof vi.fn>
    mockInvoke.mockResolvedValue([
      { node_id: 'n-1', title: 'Test', content_type: 'note', snippet: 'snippet text', score: 0.85 },
    ])

    render(<MobileSearchView />)

    fireEvent.change(screen.getByPlaceholderText('Search your knowledge graph...'), {
      target: { value: 'test query' },
    })
    fireEvent.click(screen.getByRole('button', { name: '' }))

    await vi.waitFor(() => {
      expect(screen.getByText('Test')).toBeInTheDocument()
      expect(screen.getByText('85%')).toBeInTheDocument()
    })
  })

  it('searches on Enter key', async () => {
    const { invoke } = await import('@tauri-apps/api/core')
    const mockInvoke = invoke as ReturnType<typeof vi.fn>
    mockInvoke.mockResolvedValue([])

    render(<MobileSearchView />)

    const input = screen.getByPlaceholderText('Search your knowledge graph...')
    fireEvent.change(input, { target: { value: 'test' } })
    fireEvent.keyDown(input, { key: 'Enter' })

    expect(mockInvoke).toHaveBeenCalledWith('semantic_search', { query: { query: 'test', limit: 20 } })
  })

  it('shows error message on failed search', async () => {
    const { invoke } = await import('@tauri-apps/api/core')
    const mockInvoke = invoke as ReturnType<typeof vi.fn>
    mockInvoke.mockRejectedValue(new Error('Search failed'))

    render(<MobileSearchView />)

    fireEvent.change(screen.getByPlaceholderText('Search your knowledge graph...'), {
      target: { value: 'test' },
    })
    fireEvent.click(screen.getByRole('button', { name: '' }))

    await vi.waitFor(() => {
      expect(screen.getByText(/Search failed/)).toBeInTheDocument()
    })
  })

  it('shows no results message', async () => {
    const { invoke } = await import('@tauri-apps/api/core')
    const mockInvoke = invoke as ReturnType<typeof vi.fn>
    mockInvoke.mockResolvedValue([])

    render(<MobileSearchView />)

    fireEvent.change(screen.getByPlaceholderText('Search your knowledge graph...'), {
      target: { value: 'nothing' },
    })
    fireEvent.click(screen.getByRole('button', { name: '' }))

    await vi.waitFor(() => {
      expect(screen.getByText('No results found')).toBeInTheDocument()
    })
  })
})
