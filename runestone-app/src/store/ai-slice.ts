import type { StateCreator } from 'zustand'
import type { ChatMessage, ChatResponse, Citation, SearchResult, TagSuggestion } from '../lib/types'
import * as api from '../lib/api'

export interface AISlice {
  nodeSummary: string | null
  summaryLoading: boolean
  suggestedLinks: SearchResult[]
  showChat: boolean
  chatMessages: ChatMessage[]
  chatLoading: boolean
  chatCitations: Citation[]
  chatAnswer: string | null
  tagSuggestions: TagSuggestion[]
  summarizeNode: (nodeId: string) => Promise<void>
  loadSuggestedLinks: (nodeId: string) => Promise<void>
  sendChatMessage: (question: string) => Promise<void>
  suggestTags: (nodeId: string) => Promise<void>
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export const createAISlice: StateCreator<any, [], [], AISlice> = (set, get) => ({
  nodeSummary: null,
  summaryLoading: false,
  suggestedLinks: [],
  showChat: false,
  chatMessages: [],
  chatLoading: false,
  chatCitations: [],
  chatAnswer: null,
  tagSuggestions: [],

  summarizeNode: async (nodeId: string) => {
    set({ summaryLoading: true, error: null })
    try {
      const summary = await api.summarizeNode(nodeId)
      set({ nodeSummary: summary, summaryLoading: false })
    } catch (e) {
      set({ error: `Summarization failed: ${e}`, summaryLoading: false })
    }
  },

  loadSuggestedLinks: async (nodeId: string) => {
    try {
      const results = await api.findSimilar(nodeId, 5)
      set({ suggestedLinks: results })
    } catch (e) {
      console.error('Failed to load suggestions:', e)
    }
  },

  sendChatMessage: async (question: string) => {
    const { selectedVaultId, chatMessages } = get()
    if (!selectedVaultId) return

    const userMsg: ChatMessage = { role: 'user', content: question }
    set({ chatMessages: [...chatMessages, userMsg], chatLoading: true, error: null })

    try {
      const response: ChatResponse = await api.chatWithGraph({
        vault_id: selectedVaultId,
        question,
        history: chatMessages.slice(-10),
      })

      const assistantMsg: ChatMessage = { role: 'assistant', content: response.answer }
      set((s: { chatMessages: ChatMessage[] }) => ({
        chatMessages: [...s.chatMessages, assistantMsg],
        chatCitations: response.citations,
        chatAnswer: response.answer,
        chatLoading: false,
      }))
    } catch (e) {
      set({ error: `Chat failed: ${e}`, chatLoading: false })
    }
  },

  suggestTags: async (nodeId: string) => {
    try {
      const tags = await api.suggestTags(nodeId)
      set({ tagSuggestions: tags })
    } catch (e) {
      console.error('Tag suggestion failed:', e)
    }
  },
})
