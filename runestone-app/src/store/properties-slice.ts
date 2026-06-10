import type { StateCreator } from 'zustand'
import type { NodeProperty } from '../lib/types'
import type { AppStore } from './index'
import * as api from '../lib/api'

export interface PropertiesSlice {
  nodeProperties: NodeProperty[]
  loadNodeProperties: (nodeId: string) => Promise<void>
  setProperty: (nodeId: string, key: string, value: unknown) => Promise<void>
  removeProperty: (nodeId: string, key: string) => Promise<void>
}

export const createPropertiesSlice: StateCreator<AppStore, [], [], PropertiesSlice> = (set) => ({
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
