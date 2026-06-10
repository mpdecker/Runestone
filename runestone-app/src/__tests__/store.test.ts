import { describe, it, expect, beforeEach, vi } from 'vitest'
import { useStore } from '@/store'

const mockApi = vi.hoisted(() => ({
  getConnectionStatus: vi.fn().mockResolvedValue({
    mode: 'local',
    api_url: null,
    connected: true,
    local_db_available: true,
  }),
  testConnection: vi.fn().mockResolvedValue(true),
  initDatabase: vi.fn().mockResolvedValue('ok'),
  listVaults: vi.fn().mockResolvedValue([]),
  listNodes: vi.fn().mockResolvedValue([]),
  getNode: vi.fn().mockResolvedValue({
    id: 'node-1',
    vault_id: 'vault-1',
    title: 'Test Node',
    content: '<p>Hello</p>',
    content_type: 'note',
    file_path: null,
    metadata: {},
    word_count: 1,
    created_at: null,
    updated_at: null,
  }),
  createNode: vi.fn().mockResolvedValue({
    id: 'new-node',
    vault_id: 'vault-1',
    title: 'New Node',
    content: '',
    content_type: 'note',
    file_path: null,
    metadata: {},
    word_count: 0,
    created_at: null,
    updated_at: null,
  }),
  createVault: vi.fn().mockResolvedValue({
    id: 'vault-1',
    name: 'Test',
    root_path: '/test',
    created_at: null,
    updated_at: null,
  }),
  updateNode: vi.fn().mockImplementation((req) =>
    Promise.resolve({
      id: req.id,
      vault_id: 'vault-1',
      title: 'Test Node',
      content: req.content,
      content_type: 'note',
      file_path: null,
      metadata: {},
      word_count: 1,
      created_at: null,
      updated_at: null,
    })
  ),
  deleteNode: vi.fn().mockResolvedValue(undefined),
  getGraphData: vi.fn().mockResolvedValue({ nodes: [], edges: [] }),
  getLocalGraph: vi.fn().mockResolvedValue({ nodes: [], edges: [] }),
  getBacklinks: vi.fn().mockResolvedValue([]),
  parseWikiLinks: vi.fn().mockResolvedValue([]),
  hybridSearch: vi.fn().mockResolvedValue({
    vector_results: [],
    fts_results: [],
    combined: [],
  }),
  findSimilar: vi.fn().mockResolvedValue([]),
  importDocument: vi.fn().mockResolvedValue({
    id: 'doc-1',
    vault_id: 'vault-1',
    title: 'Doc',
    content: 'test',
    content_type: 'document',
    file_path: null,
    metadata: {},
    word_count: 1,
    created_at: null,
    updated_at: null,
  }),
  extractFromDocument: vi.fn().mockResolvedValue([]),
  getPendingExtractions: vi.fn().mockResolvedValue([]),
  approveExtraction: vi.fn().mockResolvedValue(undefined),
  rejectExtraction: vi.fn().mockResolvedValue(undefined),
  batchApproveExtractions: vi.fn().mockResolvedValue(undefined),
  summarizeNode: vi.fn().mockResolvedValue('A summary'),
  chatWithGraph: vi.fn().mockResolvedValue({
    answer: 'Response',
    citations: [],
  }),
  suggestTags: vi.fn().mockResolvedValue([]),
  importObsidianVault: vi.fn().mockResolvedValue({
    nodes_created: 5,
    links_created: 3,
    files_scanned: 10,
  }),
  scanVault: vi.fn().mockResolvedValue({ created: 0, updated: 0, skipped: 0, deleted: 0 }),
  startVaultWatcher: vi.fn().mockResolvedValue(undefined),
  stopVaultWatcher: vi.fn().mockResolvedValue(undefined),
}))

vi.mock('@/lib/api', () => ({ ...mockApi }))

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

describe('useStore - state mutations (synchronous)', () => {
  beforeEach(() => {
    resetStore()
  })

  it('setFilterText updates filter text', () => {
    useStore.getState().setFilterText('hello')
    expect(useStore.getState().filterText).toBe('hello')
  })

  it('setSearchQuery updates search query', () => {
    useStore.getState().setSearchQuery('test query')
    expect(useStore.getState().searchQuery).toBe('test query')
  })

  it('toggleFilterType adds a type', () => {
    useStore.getState().toggleFilterType('note')
    expect(useStore.getState().filterTypes).toEqual(['note'])
  })

  it('toggleFilterType removes an existing type', () => {
    useStore.setState({ filterTypes: ['note', 'concept'] })
    useStore.getState().toggleFilterType('note')
    expect(useStore.getState().filterTypes).toEqual(['concept'])
  })

  it('toggleDarkMode toggles dark mode', () => {
    expect(useStore.getState().darkMode).toBe(false)
    useStore.getState().toggleDarkMode()
    expect(useStore.getState().darkMode).toBe(true)
    useStore.getState().toggleDarkMode()
    expect(useStore.getState().darkMode).toBe(false)
  })

  it('toggleSearch toggles search panel', () => {
    expect(useStore.getState().showSearch).toBe(false)
    useStore.getState().toggleSearch()
    expect(useStore.getState().showSearch).toBe(true)
    useStore.getState().toggleSearch()
    expect(useStore.getState().showSearch).toBe(false)
  })

  it('toggleCommandPalette toggles command palette', () => {
    expect(useStore.getState().showCommandPalette).toBe(false)
    useStore.getState().toggleCommandPalette()
    expect(useStore.getState().showCommandPalette).toBe(true)
    useStore.getState().toggleCommandPalette()
    expect(useStore.getState().showCommandPalette).toBe(false)
  })

  it('toggleChat toggles chat panel', () => {
    expect(useStore.getState().showChat).toBe(false)
    useStore.getState().toggleChat()
    expect(useStore.getState().showChat).toBe(true)
    useStore.getState().toggleChat()
    expect(useStore.getState().showChat).toBe(false)
  })

  it('toggleExtractions toggles and loads extractions', () => {
    useStore.setState({ selectedVaultId: 'v-1' })
    useStore.getState().toggleExtractions()
    expect(useStore.getState().showExtractions).toBe(true)
    expect(mockApi.getPendingExtractions).toHaveBeenCalledWith('v-1')
    useStore.getState().toggleExtractions()
    expect(useStore.getState().showExtractions).toBe(false)
  })

  it('toggleBacklinks loads backlinks when showing', () => {
    useStore.setState({ selectedNodeId: 'n-1' })
    useStore.getState().toggleBacklinks()
    expect(useStore.getState().showBacklinks).toBe(true)
    expect(mockApi.getBacklinks).toHaveBeenCalledWith('n-1')
  })

  it('toggleBacklinks hides without loading', () => {
    useStore.setState({ showBacklinks: true })
    useStore.getState().toggleBacklinks()
    expect(useStore.getState().showBacklinks).toBe(false)
  })

  it('setGraphViewMode global loads graph data', () => {
    useStore.setState({ selectedVaultId: 'v-1', graphViewMode: 'local' })
    useStore.getState().setGraphViewMode('global')
    expect(useStore.getState().graphViewMode).toBe('global')
    expect(mockApi.getGraphData).toHaveBeenCalledWith('v-1', undefined)
  })

  it('setGraphViewMode local does not load graph', () => {
    useStore.getState().setGraphViewMode('local')
    expect(useStore.getState().graphViewMode).toBe('local')
  })
})

describe('useStore - async actions', () => {
  beforeEach(() => {
    resetStore()
    vi.clearAllMocks()
  })

  it('initDb calls initDatabase', async () => {
    await useStore.getState().initDb()
    expect(mockApi.initDatabase).toHaveBeenCalled()
  })

  it('initDb handles error', async () => {
    const consoleError = vi.spyOn(console, 'error').mockImplementation(() => {})
    mockApi.initDatabase.mockRejectedValueOnce(new Error('fail'))
    await useStore.getState().initDb()
    expect(useStore.getState().error).toContain('fail')
    consoleError.mockRestore()
  })

  it('loadVaults fetches and sets vaults', async () => {
    const vaults = [{ id: 'v-1', name: 'Test', root_path: '/t', created_at: null, updated_at: null }]
    mockApi.listVaults.mockResolvedValueOnce(vaults)
    await useStore.getState().loadVaults()
    expect(useStore.getState().vaults).toEqual(vaults)
    expect(useStore.getState().isLoading).toBe(false)
  })

  it('loadVaults handles error', async () => {
    mockApi.listVaults.mockRejectedValueOnce(new Error('fail'))
    await useStore.getState().loadVaults()
    expect(useStore.getState().error).toContain('fail')
    expect(useStore.getState().isLoading).toBe(false)
  })

  it('selectVault sets state and triggers loads', async () => {
    useStore.setState({ selectedVaultId: null })
    const { graphData } = useStore.getState()
    expect(graphData).toBeNull()
    await useStore.getState().selectVault('v-1')
    expect(useStore.getState().selectedVaultId).toBe('v-1')
    expect(useStore.getState().selectedNodeId).toBeNull()
    expect(useStore.getState().currentNode).toBeNull()
  })

  it('selectNode fetches and sets node', async () => {
    const node = {
      id: 'n-1',
      vault_id: 'v-1',
      title: 'Test',
      content: '<p>hi</p>',
      content_type: 'note',
      file_path: null,
      metadata: {},
      word_count: 1,
      created_at: null,
      updated_at: null,
    }
    mockApi.getNode.mockResolvedValueOnce(node)
    await useStore.getState().selectNode('n-1')
    expect(useStore.getState().selectedNodeId).toBe('n-1')
    expect(useStore.getState().currentNode).toEqual(node)
    expect(useStore.getState().isEditorDirty).toBe(false)
  })

  it('updateNodeContent sets dirty flag', () => {
    useStore.setState({
      currentNode: {
        id: 'n-1',
        vault_id: 'v-1',
        title: 'Test',
        content: '',
        content_type: 'note',
        file_path: null,
        metadata: {},
        word_count: 0,
        created_at: null,
        updated_at: null,
      },
    })
    useStore.getState().updateNodeContent('<p>new</p>')
    expect(useStore.getState().currentNode?.content).toBe('<p>new</p>')
    expect(useStore.getState().isEditorDirty).toBe(true)
  })

  it('updateNodeContent does nothing if no currentNode', () => {
    useStore.getState().updateNodeContent('test')
    expect(useStore.getState().currentNode).toBeNull()
    expect(useStore.getState().isEditorDirty).toBe(false)
  })

  it('saveNode calls updateNode and clears dirty', async () => {
    useStore.setState({
      currentNode: {
        id: 'n-1',
        vault_id: 'v-1',
        title: 'Test',
        content: '<p>hello</p>',
        content_type: 'note',
        file_path: null,
        metadata: {},
        word_count: 0,
        created_at: null,
        updated_at: null,
      },
      isEditorDirty: true,
    })
    await useStore.getState().saveNode()
    expect(mockApi.updateNode).toHaveBeenCalledWith({
      id: 'n-1',
      content: '<p>hello</p>',
    })
    expect(useStore.getState().isEditorDirty).toBe(false)
  })

  it('saveNode does nothing if no currentNode', async () => {
    await useStore.getState().saveNode()
    expect(mockApi.updateNode).not.toHaveBeenCalled()
  })

  it('deleteNode removes node and refreshes', async () => {
    useStore.setState({ selectedVaultId: 'v-1' })
    await useStore.getState().deleteNode('n-1')
    expect(mockApi.deleteNode).toHaveBeenCalledWith('n-1')
    expect(useStore.getState().selectedNodeId).toBeNull()
    expect(useStore.getState().currentNode).toBeNull()
    expect(mockApi.listNodes).toHaveBeenCalledWith('v-1')
  })

  it('runSearch does nothing if no vault', async () => {
    await useStore.getState().runSearch()
    expect(mockApi.hybridSearch).not.toHaveBeenCalled()
  })

  it('runSearch does nothing for empty query', async () => {
    useStore.setState({ selectedVaultId: 'v-1', searchQuery: '  ' })
    await useStore.getState().runSearch()
    expect(mockApi.hybridSearch).not.toHaveBeenCalled()
  })

  it('runSearch performs search', async () => {
    useStore.setState({ selectedVaultId: 'v-1', searchQuery: 'test' })
    const results = { vector_results: [], fts_results: [], combined: [] }
    mockApi.hybridSearch.mockResolvedValueOnce(results)
    await useStore.getState().runSearch()
    expect(mockApi.hybridSearch).toHaveBeenCalledWith({
      vault_id: 'v-1',
      query: 'test',
      limit: 20,
      include_fts: true,
    })
    expect(useStore.getState().searchResults).toEqual(results)
    expect(useStore.getState().showSearch).toBe(true)
  })

  it('findSimilar searches by node id', async () => {
    const results = [
      { node_id: 'n-2', title: 'Similar', content_type: 'note', snippet: 's', score: 0.9 },
    ]
    mockApi.findSimilar.mockResolvedValueOnce(results)
    await useStore.getState().findSimilar('n-1')
    expect(mockApi.findSimilar).toHaveBeenCalledWith('n-1', 10)
    expect(useStore.getState().similarNodes).toEqual(results)
  })

  it('importDocument requires selected vault', async () => {
    await useStore.getState().importDocument('/file.pdf')
    expect(mockApi.importDocument).not.toHaveBeenCalled()
  })

  it('importDocument imports and refreshes', async () => {
    useStore.setState({ selectedVaultId: 'v-1' })
    await useStore.getState().importDocument('/file.pdf')
    expect(mockApi.importDocument).toHaveBeenCalledWith('v-1', '/file.pdf')
    expect(mockApi.listNodes).toHaveBeenCalledWith('v-1')
    expect(useStore.getState().isLoading).toBe(false)
  })

  it('extractFromDocument extracts', async () => {
    const extractions = [
      {
        name: 'Entity', description: 'Test', extraction_type: 'entity',
        confidence: 0.8, source_node_id: 'n-1', chunk_index: 0,
      },
    ]
    mockApi.extractFromDocument.mockResolvedValueOnce(extractions)
    await useStore.getState().extractFromDocument('n-1')
    expect(useStore.getState().extractionResults).toEqual(extractions)
    expect(useStore.getState().isLoading).toBe(false)
  })

  it('approveExtraction removes from pending', async () => {
    useStore.setState({
      pendingExtractions: [
        { id: 'e-1', title: 'E1', content_type: 'entity', metadata: null, created_at: null },
        { id: 'e-2', title: 'E2', content_type: 'concept', metadata: null, created_at: null },
      ],
    })
    await useStore.getState().approveExtraction('e-1')
    expect(mockApi.approveExtraction).toHaveBeenCalledWith('e-1')
    expect(useStore.getState().pendingExtractions).toHaveLength(1)
    expect(useStore.getState().pendingExtractions[0].id).toBe('e-2')
  })

  it('rejectExtraction removes from pending', async () => {
    useStore.setState({
      pendingExtractions: [{ id: 'e-1', title: 'E1', content_type: 'entity', metadata: null, created_at: null }],
    })
    await useStore.getState().rejectExtraction('e-1')
    expect(useStore.getState().pendingExtractions).toHaveLength(0)
  })

  it('batchApproveExtractions removes all selected', async () => {
    useStore.setState({
      pendingExtractions: [
        { id: 'e-1', title: 'E1', content_type: 'entity', metadata: null, created_at: null },
        { id: 'e-2', title: 'E2', content_type: 'concept', metadata: null, created_at: null },
        { id: 'e-3', title: 'E3', content_type: 'entity', metadata: null, created_at: null },
      ],
    })
    await useStore.getState().batchApproveExtractions(['e-1', 'e-3'])
    expect(useStore.getState().pendingExtractions).toHaveLength(1)
    expect(useStore.getState().pendingExtractions[0].id).toBe('e-2')
  })

  it('summarizeNode fetches summary', async () => {
    mockApi.summarizeNode.mockResolvedValueOnce('This is a summary')
    await useStore.getState().summarizeNode('n-1')
    expect(useStore.getState().nodeSummary).toBe('This is a summary')
    expect(useStore.getState().summaryLoading).toBe(false)
  })

  it('sendChatMessage sends message and appends response', async () => {
    useStore.setState({ selectedVaultId: 'v-1' })
    mockApi.chatWithGraph.mockResolvedValueOnce({
      answer: 'Hello back',
      citations: [{ node_id: 'n-1', title: 'N1', snippet: 's' }],
    })
    await useStore.getState().sendChatMessage('Hello')
    expect(useStore.getState().chatMessages).toHaveLength(2)
    expect(useStore.getState().chatMessages[0]).toEqual({ role: 'user', content: 'Hello' })
    expect(useStore.getState().chatMessages[1].role).toBe('assistant')
    expect(useStore.getState().chatAnswer).toBe('Hello back')
    expect(useStore.getState().chatCitations).toEqual([{ node_id: 'n-1', title: 'N1', snippet: 's' }])
  })

  it('sendChatMessage does nothing without vault', async () => {
    await useStore.getState().sendChatMessage('Hello')
    expect(mockApi.chatWithGraph).not.toHaveBeenCalled()
  })

  it('createNode creates and refreshes', async () => {
    useStore.setState({ selectedVaultId: 'v-1' })
    await useStore.getState().createNode('My Note')
    expect(mockApi.createNode).toHaveBeenCalledWith({
      vault_id: 'v-1',
      title: 'My Note',
      content: '',
    })
    expect(mockApi.listNodes).toHaveBeenCalledWith('v-1')
  })

  it('createNode handles no vault', async () => {
    await useStore.getState().createNode('Test')
    expect(mockApi.createNode).not.toHaveBeenCalled()
  })

  it('createVault creates and refreshes', async () => {
    await useStore.getState().createVault('V1', '/path')
    expect(mockApi.createVault).toHaveBeenCalledWith({ name: 'V1', root_path: '/path' })
    expect(mockApi.listVaults).toHaveBeenCalled()
  })

  it('importObsidian imports and refreshes', async () => {
    useStore.setState({ selectedVaultId: 'v-1' })
    await useStore.getState().importObsidian('/vault')
    expect(mockApi.importObsidianVault).toHaveBeenCalledWith('v-1', '/vault')
    expect(useStore.getState().importResult).toBeDefined()
    expect(mockApi.listNodes).toHaveBeenCalledWith('v-1')
  })

  it('importObsidian handles no vault', async () => {
    await useStore.getState().importObsidian('/vault')
    expect(mockApi.importObsidianVault).not.toHaveBeenCalled()
  })

  it('loadNodes handles no vault', async () => {
    await useStore.getState().loadNodes()
    expect(mockApi.listNodes).not.toHaveBeenCalled()
  })

  it('scanVault handles no vault', async () => {
    await useStore.getState().scanVault()
    expect(mockApi.scanVault).not.toHaveBeenCalled()
  })
})
