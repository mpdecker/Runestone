import type { StateCreator } from 'zustand'
import type { PropertiesResponse, NodeProperty } from '../lib/types'
import * as api from '../lib/api'

export interface PropertiesSlice {
  nodeProperties: NodeProperty[]
  loadNodeProperties: (nodeId: string) => Promise<void>
  setProperty: (nodeId: string, key: string, value: unknown) => Promise<void>
  removeProperty: (nodeId: string, key: string) => Promise<void>
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export const createPropertiesSlice: StateCreator<any, [], [], PropertiesSlice> = (set, get) => ({
  nodeProperties: [],

  loadNodeProperties: async (nodeId: string) => {
    try {
      const response = await api.getNodeProperties(nodeId)
      set({ nodeProperties: response.properties })
    } catch (e) {
      console.error('Failed to load properties:', e)
    }
  },

  setProperty: async (nodeId: string, key: string, value: unknown) => {
    try {
      const response = await api.setNodeProperty({ node_id: nodeId, key, value })
      set({ nodeProperties: response.properties })
    } catch (e) {
      console.error('Failed to set property:', e)
    }
  },

  removeProperty: async (nodeId: string, key: string) => {
    try {
      const response = await api.removeNodeProperty(nodeId, key)
      set({ nodeProperties: response.properties })
    } catch (e) {
      console.error('Failed to remove property:', e)
    }
  },
})
