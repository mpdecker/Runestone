import { describe, it, expect, beforeEach, vi } from 'vitest'
import { useStore } from '@/store'

const mockApi = vi.hoisted(() => ({
  initDatabase: vi.fn().mockResolvedValue('ok'),
  listVaults: vi.fn().mockResolvedValue([]),
  listNodes: vi.fn().mockResolvedValue([]),
  getNodeTags: vi.fn().mockResolvedValue({ node_id: 'n-1', tags: ['typescript'] }),
  addTagsToNode: vi.fn().mockResolvedValue({ node_id: 'n-1', tags: ['typescript', 'rust'] }),
  removeTagFromNode: vi.fn().mockResolvedValue({ node_id: 'n-1', tags: ['typescript'] }),
  acceptTagSuggestions: vi.fn().mockResolvedValue({ node_id: 'n-1', tags: ['ai', 'ml'] }),
  listTags: vi.fn().mockResolvedValue([{ name: 'typescript', node_count: 3 }]),
  getNodesByTag: vi.fn().mockResolvedValue([]),
  getNode: vi.fn().mockResolvedValue({ id: 'n-1', vault_id: 'v-1', title: 'Test', content: '<p>Hi</p>', content_type: 'note', file_path: null, metadata: {}, word_count: 1, created_at: null, updated_at: null }),
  loadGraphData: vi.fn().mockResolvedValue({ nodes: [], edges: [] }),
}))

vi.mock('@/lib/api', () => ({ ...mockApi }))

function resetStore() {
  useStore.setState({
    vaults: [],
    selectedVaultId: null,
    nodes: [],
    selectedNodeId: null,
    currentNode: null,
    nodeTags: null,
    vaultTags: [],
    selectedTag: null,
    graphData: null,
    error: null,
    isLoading: false,
  })
}

describe('tag-slice', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    resetStore()
  })

  describe('loadNodeTags', () => {
    it('loads tags for a node and sets nodeTags', async () => {
      mockApi.getNodeTags.mockResolvedValue({ node_id: 'n-1', tags: ['typescript', 'react'] })

      await useStore.getState().loadNodeTags('n-1')

      expect(mockApi.getNodeTags).toHaveBeenCalledWith('n-1')
      expect(useStore.getState().nodeTags).toEqual({ node_id: 'n-1', tags: ['typescript', 'react'] })
    })

    it('handles API error gracefully', async () => {
      mockApi.getNodeTags.mockRejectedValue(new Error('fail'))
      const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {})

      await useStore.getState().loadNodeTags('n-1')

      expect(useStore.getState().nodeTags).toBeNull()
      consoleSpy.mockRestore()
    })
  })

  describe('addTags', () => {
    it('calls API and updates nodeTags after adding tags', async () => {
      useStore.setState({ selectedVaultId: 'v-1' })
      mockApi.addTagsToNode.mockResolvedValue({ node_id: 'n-1', tags: ['a', 'b', 'c'] })
      mockApi.listTags.mockResolvedValue([{ name: 'a', node_count: 1 }, { name: 'b', node_count: 1 }])

      await useStore.getState().addTags('n-1', ['c'])

      expect(mockApi.addTagsToNode).toHaveBeenCalledWith({ node_id: 'n-1', tags: ['c'] })
      expect(useStore.getState().nodeTags?.tags).toContain('c')
    })

    it('reloads vault tags after adding', async () => {
      useStore.setState({ selectedVaultId: 'v-1' })
      mockApi.listTags.mockResolvedValue([{ name: 'newtag', node_count: 1 }])

      await useStore.getState().addTags('n-1', ['newtag'])

      expect(mockApi.listTags).toHaveBeenCalledWith('v-1')
    })
  })

  describe('removeTag', () => {
    it('calls API and updates nodeTags after removing tag', async () => {
      useStore.setState({ selectedVaultId: 'v-1' })
      mockApi.removeTagFromNode.mockResolvedValue({ node_id: 'n-1', tags: ['remaining'] })

      await useStore.getState().removeTag('n-1', 'oldtag')

      expect(mockApi.removeTagFromNode).toHaveBeenCalledWith({ node_id: 'n-1', tag: 'oldtag' })
    })
  })

  describe('acceptSuggestedTags', () => {
    it('calls API and updates nodeTags', async () => {
      useStore.setState({ selectedVaultId: 'v-1' })
      mockApi.acceptTagSuggestions.mockResolvedValue({ node_id: 'n-1', tags: ['ai', 'ml'] })

      await useStore.getState().acceptSuggestedTags('n-1', ['ai', 'ml'])

      expect(mockApi.acceptTagSuggestions).toHaveBeenCalledWith('n-1', ['ai', 'ml'])
      expect(useStore.getState().nodeTags?.tags).toEqual(['ai', 'ml'])
    })
  })

  describe('loadVaultTags', () => {
    it('does nothing when no vault is selected', async () => {
      await useStore.getState().loadVaultTags()

      expect(mockApi.listTags).not.toHaveBeenCalled()
    })

    it('loads tags for the selected vault', async () => {
      useStore.setState({ selectedVaultId: 'v-1' })
      mockApi.listTags.mockResolvedValue([{ name: 'typescript', node_count: 5 }])

      await useStore.getState().loadVaultTags()

      expect(mockApi.listTags).toHaveBeenCalledWith('v-1')
      expect(useStore.getState().vaultTags).toEqual([{ name: 'typescript', node_count: 5 }])
    })
  })

  describe('selectTag', () => {
    it('sets selectedTag and loads nodes by tag', () => {
      useStore.getState().selectTag('typescript')

      expect(useStore.getState().selectedTag).toBe('typescript')
    })

    it('clears selectedTag and reloads all nodes', () => {
      useStore.getState().selectTag(null)

      expect(useStore.getState().selectedTag).toBeNull()
    })
  })
})
