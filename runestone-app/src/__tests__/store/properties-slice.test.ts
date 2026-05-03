import { describe, it, expect, beforeEach, vi } from 'vitest'
import { useStore } from '@/store'

const mockApi = vi.hoisted(() => ({
  initDatabase: vi.fn().mockResolvedValue('ok'),
  listVaults: vi.fn().mockResolvedValue([]),
  getNodeProperties: vi.fn().mockResolvedValue({
    node_id: 'n-1',
    properties: [{ key: 'priority', value: 'high', prop_type: 'text' }],
  }),
  setNodeProperty: vi.fn().mockResolvedValue({
    node_id: 'n-1',
    properties: [{ key: 'priority', value: 'high', prop_type: 'text' }],
  }),
  removeNodeProperty: vi.fn().mockResolvedValue({
    node_id: 'n-1',
    properties: [],
  }),
}))

vi.mock('@/lib/api', () => ({ ...mockApi }))

function resetStore() {
  useStore.setState({
    vaults: [],
    selectedVaultId: null,
    selectedNodeId: null,
    nodeProperties: [],
    error: null,
    isLoading: false,
  })
}

describe('properties-slice', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    resetStore()
  })

  describe('loadNodeProperties', () => {
    it('loads properties for a node', async () => {
      const props = { node_id: 'n-1', properties: [{ key: 'color', value: 'blue', prop_type: 'text' }] }
      mockApi.getNodeProperties.mockResolvedValue(props)

      await useStore.getState().loadNodeProperties('n-1')

      expect(mockApi.getNodeProperties).toHaveBeenCalledWith('n-1')
      expect(useStore.getState().nodeProperties).toEqual(props.properties)
    })

    it('handles API error gracefully', async () => {
      mockApi.getNodeProperties.mockRejectedValue(new Error('fail'))
      const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {})

      await useStore.getState().loadNodeProperties('n-1')

      expect(useStore.getState().nodeProperties).toEqual([])
      consoleSpy.mockRestore()
    })
  })

  describe('setProperty', () => {
    it('calls API and reloads properties', async () => {
      mockApi.setNodeProperty.mockResolvedValue({
        node_id: 'n-1',
        properties: [{ key: 'status', value: 'done', prop_type: 'text' }],
      })

      await useStore.getState().setProperty('n-1', 'status', 'done')

      expect(mockApi.setNodeProperty).toHaveBeenCalledWith({
        node_id: 'n-1',
        key: 'status',
        value: 'done',
      })
      expect(useStore.getState().nodeProperties).toEqual([{ key: 'status', value: 'done', prop_type: 'text' }])
    })
  })

  describe('removeProperty', () => {
    it('calls API and reloads properties', async () => {
      mockApi.removeNodeProperty.mockResolvedValue({
        node_id: 'n-1',
        properties: [],
      })

      await useStore.getState().removeProperty('n-1', 'oldkey')

      expect(mockApi.removeNodeProperty).toHaveBeenCalledWith('n-1', 'oldkey')
      expect(useStore.getState().nodeProperties).toEqual([])
    })
  })
})
