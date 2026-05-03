import { create } from 'zustand'
import { createVaultSlice, type VaultSlice } from './vault-slice'
import { createNodeSlice, type NodeSlice } from './node-slice'
import { createGraphSlice, type GraphSlice } from './graph-slice'
import { createSearchSlice, type SearchSlice } from './search-slice'
import { createExtractionSlice, type ExtractionSlice } from './extraction-slice'
import { createAISlice, type AISlice } from './ai-slice'
import { createUISlice, type UISlice } from './ui-slice'
import { createObsidianSlice, type ObsidianSlice } from './obsidian-slice'
import { createTagSlice, type TagSlice } from './tag-slice'
import { createPropertiesSlice, type PropertiesSlice } from './properties-slice'
import { createTabSlice, type TabSlice } from './tab-slice'
import { createPluginSlice, type PluginSlice } from './plugin-slice'

export type AppStore = VaultSlice & NodeSlice & GraphSlice & SearchSlice & ExtractionSlice & AISlice & UISlice & ObsidianSlice & TagSlice & PropertiesSlice & TabSlice & PluginSlice

export const useStore = create<AppStore>()((...args) => ({
  ...createVaultSlice(...args),
  ...createNodeSlice(...args),
  ...createGraphSlice(...args),
  ...createSearchSlice(...args),
  ...createExtractionSlice(...args),
  ...createAISlice(...args),
  ...createUISlice(...args),
  ...createObsidianSlice(...args),
  ...createTagSlice(...args),
  ...createPropertiesSlice(...args),
  ...createTabSlice(...args),
  ...createPluginSlice(...args),
}))
