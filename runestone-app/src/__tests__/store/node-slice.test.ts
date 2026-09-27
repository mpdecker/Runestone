import { describe, it, expect, beforeEach, vi } from 'vitest'
import { useStore } from '@/store'

const mockApi = vi.hoisted(() => ({
  listNodes: vi
    .fn()
    .mockResolvedValue([
      { id: 'n-1', title: 'Note', content_type: 'note', file_path: null, updated_at: null },
    ]),
  getNode: vi.fn().mockResolvedValue({
    id: 'n-1',
    vault_id: 'v-1',
    title: 'Note',
    content: '<p>Hi</p>',
    content_type: 'note',
    file_path: null,
    metadata: {},
    word_count: 1,
    created_at: null,
    updated_at: null,
  }),
  createNode: vi.fn().mockResolvedValue({
    id: 'n-2',
    vault_id: 'v-1',
    title: 'New',
    content: '',
    content_type: 'note',
    file_path: null,
    metadata: {},
    word_count: 0,
    created_at: null,
    updated_at: null,
  }),
  updateNode: vi.fn(),
  scanVault: vi.fn().mockResolvedValue({ created: 1, updated: 0, skipped: 0, deleted: 0 }),
  getGraphData: vi.fn().mockResolvedValue({ nodes: [], edges: [] }),
  addTab: vi.fn(),
}))

vi.mock('@/lib/api', () => ({ ...mockApi }))

function resetStore() {
  useStore.setState({
    selectedVaultId: 'v-1',
    nodes: [],
    selectedNodeId: null,
    currentNode: null,
    nodeError: null,
    nodeLoading: false,
    openTabs: [],
    activeTabId: null,
  })
}

describe('node-slice', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    resetStore()
  })

  it('loadNodes populates nodes list', async () => {
    await useStore.getState().loadNodes()
    expect(useStore.getState().nodes).toHaveLength(1)
    expect(useStore.getState().nodeLoading).toBe(false)
  })

  it('createNode selects new node', async () => {
    await useStore.getState().createNode('New')
    expect(mockApi.createNode).toHaveBeenCalled()
    expect(useStore.getState().selectedNodeId).toBe('n-2')
  })

  it('scanVault reloads nodes', async () => {
    await useStore.getState().scanVault()
    expect(mockApi.scanVault).toHaveBeenCalledWith('v-1', true)
    expect(mockApi.listNodes).toHaveBeenCalled()
  })

  const base = { id: 'n-1', vault_id: 'v-1', title: 'Note', content: '<p>Hi</p>', content_type: 'note', file_path: null, metadata: {}, word_count: 1, created_at: null, updated_at: null }

  it('selectNode saves unsaved edits of the note being left instead of dropping them', async () => {
    mockApi.updateNode.mockImplementation(async (req: { id: string; content: string }) => ({
      ...base,
      id: req.id,
      content: req.content,
    }))
    useStore.setState({
      selectedNodeId: 'n-1',
      currentNode: { ...base, content: '<p>typed</p>' },
      isEditorDirty: true,
    })
    await useStore.getState().selectNode('n-3')
    expect(mockApi.updateNode).toHaveBeenCalledWith({ id: 'n-1', content: '<p>typed</p>' })
    expect(mockApi.updateNode.mock.invocationCallOrder[0]).toBeLessThan(
      mockApi.getNode.mock.invocationCallOrder[0],
    )
  })

  it('saveNode keeps keystrokes typed while the request was in flight', async () => {
    let resolveSave: (n: unknown) => void = () => {}
    mockApi.updateNode.mockImplementation(() => new Promise((r) => (resolveSave = r)))
    useStore.setState({ currentNode: { ...base, content: '<p>a</p>' }, isEditorDirty: true })
    const saving = useStore.getState().saveNode()
    useStore.getState().updateNodeContent('<p>ab</p>')
    resolveSave({ ...base, content: '<p>a</p>' })
    await saving
    expect(useStore.getState().currentNode?.content).toBe('<p>ab</p>')
    expect(useStore.getState().isEditorDirty).toBe(true)
  })

  it('saveNode does not overwrite a different note selected during the save', async () => {
    let resolveSave: (n: unknown) => void = () => {}
    mockApi.updateNode.mockImplementation(() => new Promise((r) => (resolveSave = r)))
    useStore.setState({ currentNode: { ...base, content: '<p>a</p>' }, isEditorDirty: true })
    const saving = useStore.getState().saveNode()
    useStore.setState({ currentNode: { ...base, id: 'n-9', content: '<p>other</p>' } })
    resolveSave({ ...base, content: '<p>a</p>' })
    await saving
    expect(useStore.getState().currentNode?.id).toBe('n-9')
  })
})
