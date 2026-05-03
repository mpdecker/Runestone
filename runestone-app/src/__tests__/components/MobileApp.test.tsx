import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import { MobileApp } from '@/features/layout/MobileApp'
import { useStore } from '@/store'
import type { Mock } from 'vitest'

vi.mock('@/features/layout/MobileGraphView', () => ({
  MobileGraphView: () => <div data-testid="mobile-graph">Graph</div>,
}))
vi.mock('@/features/layout/MobileNotesList', () => ({
  MobileNotesList: () => <div data-testid="mobile-notes">Notes</div>,
}))
vi.mock('@/features/layout/MobileSearchView', () => ({
  MobileSearchView: () => <div data-testid="mobile-search">Search</div>,
}))
vi.mock('@/features/layout/MobileSettingsView', () => ({
  MobileSettingsView: () => <div data-testid="mobile-settings">Settings</div>,
}))
vi.mock('@/features/layout/MobileTabBar', () => ({
  MobileTabBar: ({ activeTab, onTabChange }: { activeTab: string; onTabChange: (tab: string) => void }) => (
    <div data-testid="mobile-tab-bar">
      <button onClick={() => onTabChange('graph')}>Graph Tab</button>
      <button onClick={() => onTabChange('notes')}>Notes Tab</button>
      <button onClick={() => onTabChange('search')}>Search Tab</button>
      <button onClick={() => onTabChange('settings')}>Settings Tab</button>
      <span data-testid="active-tab">{activeTab}</span>
    </div>
  ),
}))
vi.mock('@/features/layout/ConnectionScreen', () => ({
  ConnectionScreen: ({ onConnected }: { onConnected: () => void }) => (
    <div data-testid="connection-screen">
      <button onClick={onConnected}>Connect</button>
    </div>
  ),
}))

function resetStore() {
  useStore.setState({
    vaults: [],
    selectedVaultId: null,
    nodes: [],
    selectedNodeId: null,
    nodeProperties: [],
    openTabs: [],
    error: null,
    isLoading: false,
  })
}

describe('MobileApp', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    resetStore()
    localStorage.clear()
  })

  it('shows ConnectionScreen when no server configured', () => {
    localStorage.removeItem('runestone_server_url')
    render(<MobileApp />)

    expect(screen.getByTestId('connection-screen')).toBeInTheDocument()
    expect(screen.queryByTestId('mobile-notes')).not.toBeInTheDocument()
  })

  it('removes connection screen after server is configured', () => {
    render(<MobileApp />)

    fireEvent.click(screen.getByText('Connect'))

    expect(screen.queryByTestId('connection-screen')).not.toBeInTheDocument()
    expect(screen.getByTestId('mobile-notes')).toBeInTheDocument()
  })

  it('renders notes tab by default after connecting', () => {
    localStorage.setItem('runestone_server_url', 'http://server')
    render(<MobileApp />)

    expect(screen.getByTestId('mobile-notes')).toBeInTheDocument()
  })

  it('switches to graph tab', () => {
    localStorage.setItem('runestone_server_url', 'http://server')
    render(<MobileApp />)

    fireEvent.click(screen.getByText('Graph Tab'))

    expect(screen.getByTestId('active-tab').textContent).toBe('graph')
    expect(screen.getByTestId('mobile-graph')).toBeInTheDocument()
  })

  it('switches to search tab', () => {
    localStorage.setItem('runestone_server_url', 'http://server')
    render(<MobileApp />)

    fireEvent.click(screen.getByText('Search Tab'))

    expect(screen.getByTestId('mobile-search')).toBeInTheDocument()
  })

  it('switches to settings tab', () => {
    localStorage.setItem('runestone_server_url', 'http://server')
    render(<MobileApp />)

    fireEvent.click(screen.getByText('Settings Tab'))

    expect(screen.getByTestId('mobile-settings')).toBeInTheDocument()
  })

  it('renders MobileTabBar', () => {
    localStorage.setItem('runestone_server_url', 'http://server')
    render(<MobileApp />)

    expect(screen.getByTestId('mobile-tab-bar')).toBeInTheDocument()
  })
})
