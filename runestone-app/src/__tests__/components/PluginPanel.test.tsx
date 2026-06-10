import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import { PluginPanel } from '@/features/sidebar/PluginPanel'
import { useStore } from '@/store'
import { makePluginInfo } from '@/__tests__/helpers/fixtures'

function resetStore() {
  useStore.setState({
    vaults: [],
    selectedVaultId: null,
    pluginDir: '',
    availablePlugins: [],
    installedPlugins: [],
    pluginLoading: false,
    error: null,
  })
}

describe('PluginPanel', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    resetStore()
  })

  it('renders directory input and Scan button', () => {
    render(<PluginPanel />)
    expect(screen.getByPlaceholderText('Plugins directory path...')).toBeInTheDocument()
    expect(screen.getByText('Scan')).toBeInTheDocument()
  })

  it('calls setPluginDir and discoverPlugins on Scan click', () => {
    let dirSet = ''
    let discovered = false
    useStore.setState({
      setPluginDir: (d: string) => { dirSet = d },
      discoverPlugins: async () => { discovered = true },
    } as any)
    render(<PluginPanel />)

    fireEvent.change(screen.getByPlaceholderText('Plugins directory path...'), { target: { value: '/home/plugins' } })
    fireEvent.click(screen.getByText('Scan'))

    expect(dirSet).toBe('/home/plugins')
    expect(discovered).toBe(true)
  })

  it('shows loading state', () => {
    useStore.setState({ pluginLoading: true })
    render(<PluginPanel />)
    expect(screen.getByText('Discovering...')).toBeInTheDocument()
  })

  it('shows available plugins list', () => {
    useStore.setState({
      availablePlugins: [
        makePluginInfo({ name: 'test-plugin', path: '/p/test', main_file: 'index.js' }),
      ],
    })
    render(<PluginPanel />)
    expect(screen.getByText('test-plugin')).toBeInTheDocument()
    expect(screen.getByText('v1.0.0')).toBeInTheDocument()
  })

  it('shows Load button for uninstalled plugins', () => {
    useStore.setState({
      availablePlugins: [
        makePluginInfo({ name: 'test-plugin', path: '/p/test', main_file: 'index.js' }),
      ],
    })
    render(<PluginPanel />)
    expect(screen.getByText('Load')).toBeInTheDocument()
  })

  it('shows On/Off toggle for installed plugins', () => {
    useStore.setState({
      availablePlugins: [
        makePluginInfo({ name: 'test-plugin', path: '/p/test', main_file: 'index.js' }),
      ],
      installedPlugins: [
        {
          manifest: { name: 'test-plugin', version: '1.0.0', description: 'Test', main: 'index.js' },
          enabled: true,
          activate: () => {},
          deactivate: () => {},
        },
      ],
    })
    render(<PluginPanel />)
    expect(screen.getByText('On')).toBeInTheDocument()
  })

  it('shows Off for disabled installed plugins', () => {
    useStore.setState({
      availablePlugins: [
        makePluginInfo({ name: 'test-plugin', path: '/p/test', main_file: 'index.js' }),
      ],
      installedPlugins: [
        {
          manifest: { name: 'test-plugin', version: '1.0.0', description: 'Test', main: 'index.js' },
          enabled: false,
          activate: () => {},
          deactivate: () => {},
        },
      ],
    })
    render(<PluginPanel />)
    expect(screen.getByText('Off')).toBeInTheDocument()
  })

  it('shows empty message when no plugins found but dir is set', () => {
    useStore.setState({ pluginDir: '/empty', availablePlugins: [], pluginLoading: false })
    render(<PluginPanel />)
    expect(screen.getByText('No plugins found')).toBeInTheDocument()
  })
})
