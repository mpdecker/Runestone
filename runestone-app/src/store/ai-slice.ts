import { listen } from '@tauri-apps/api/event'
import type { StateCreator } from 'zustand'
import type { ChatMessage, ChatResponse, Citation, SearchResult, TagSuggestion } from '../lib/types'
import type { AppStore } from './index'
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

export const createAISlice: StateCreator<AppStore, [], [], AISlice> = (set, get) => ({
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
    const assistantMsg: ChatMessage = { role: 'assistant', content: '' }
    set({
      chatMessages: [...chatMessages, userMsg, assistantMsg],
      chatLoading: true,
      error: null,
      chatCitations: [],
    })

    let streamUnlisten: (() => void) | null = null
    const canStream =
      typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
    try {
      if (canStream) {
        streamUnlisten = await listen<string>('chat-stream-chunk', (event) => {
        set((s: { chatMessages: ChatMessage[] }) => {
          const msgs = [...s.chatMessages]
          const last = msgs[msgs.length - 1]
          if (last?.role === 'assistant') {
            msgs[msgs.length - 1] = { ...last, content: last.content + event.payload }
          }
          return { chatMessages: msgs }
        })
      })
      }

      const response: ChatResponse = canStream
        ? await api.chatWithGraphStream({
        vault_id: selectedVaultId,
        question,
        history: chatMessages.slice(-10),
      })
        : await api.chatWithGraph({
        vault_id: selectedVaultId,
        question,
        history: chatMessages.slice(-10),
      })

      set((s: { chatMessages: ChatMessage[] }) => {
        const msgs = [...s.chatMessages]
        const last = msgs[msgs.length - 1]
        if (last?.role === 'assistant') {
          msgs[msgs.length - 1] = { role: 'assistant', content: response.answer }
        }
        return {
          chatMessages: msgs,
          chatCitations: response.citations,
          chatAnswer: response.answer,
          chatLoading: false,
        }
      })
    } catch (e) {
      set({ error: `Chat failed: ${e}`, chatLoading: false })
    } finally {
      streamUnlisten?.()
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
