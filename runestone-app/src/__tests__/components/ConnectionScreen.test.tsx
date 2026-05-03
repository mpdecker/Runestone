import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import { ConnectionScreen } from '@/features/layout/ConnectionScreen'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

describe('ConnectionScreen', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    localStorage.clear()
  })

  it('renders server URL input', () => {
    render(<ConnectionScreen onConnected={vi.fn()} />)
    expect(screen.getByPlaceholderText('https://my-server.com')).toBeInTheDocument()
  })

  it('renders auth token input', () => {
    render(<ConnectionScreen onConnected={vi.fn()} />)
    expect(screen.getByPlaceholderText('Bearer token')).toBeInTheDocument()
  })

  it('renders Connect button', () => {
    render(<ConnectionScreen onConnected={vi.fn()} />)
    expect(screen.getByText('Connect')).toBeInTheDocument()
  })

  it('disables Connect when URL is empty', () => {
    render(<ConnectionScreen onConnected={vi.fn()} />)
    expect(screen.getByText('Connect')).toBeDisabled()
  })

  it('enables Connect when URL is entered', () => {
    render(<ConnectionScreen onConnected={vi.fn()} />)
    fireEvent.change(screen.getByPlaceholderText('https://my-server.com'), {
      target: { value: 'http://localhost:3000' },
    })
    expect(screen.getByText('Connect')).not.toBeDisabled()
  })

  it('calls invoke with URL on connect', async () => {
    const { invoke } = await import('@tauri-apps/api/core')
    const mockInvoke = invoke as ReturnType<typeof vi.fn>
    mockInvoke.mockResolvedValue(undefined)

    render(<ConnectionScreen onConnected={vi.fn()} />)

    fireEvent.change(screen.getByPlaceholderText('https://my-server.com'), {
      target: { value: 'http://server:3000' },
    })
    fireEvent.click(screen.getByText('Connect'))

    expect(mockInvoke).toHaveBeenCalledWith('configure_server_connection', {
      apiUrl: 'http://server:3000',
      authToken: null,
    })
  })

  it('shows error message on failed connection', async () => {
    const { invoke } = await import('@tauri-apps/api/core')
    const mockInvoke = invoke as ReturnType<typeof vi.fn>
    mockInvoke.mockRejectedValue(new Error('Connection refused'))

    render(<ConnectionScreen onConnected={vi.fn()} />)

    fireEvent.change(screen.getByPlaceholderText('https://my-server.com'), {
      target: { value: 'http://bad-server' },
    })
    fireEvent.click(screen.getByText('Connect'))

    await vi.waitFor(() => {
      expect(screen.getByText(/Connection refused/)).toBeInTheDocument()
    })
  })
})
