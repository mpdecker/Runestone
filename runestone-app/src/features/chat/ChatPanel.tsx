import { useState } from 'react'
import { useShallow } from 'zustand/react/shallow'
import { useStore } from '@/store'
import type { ChatMessage, Citation } from '@/lib/types'

export function ChatPanel() {
  const {
    selectedVaultId, showChat, toggleChat,
    chatMessages, chatLoading, sendChatMessage,
    chatCitations, selectNode,
  } = useStore(
    useShallow((s) => ({
      selectedVaultId: s.selectedVaultId,
      showChat: s.showChat,
      toggleChat: s.toggleChat,
      chatMessages: s.chatMessages,
      chatLoading: s.chatLoading,
      sendChatMessage: s.sendChatMessage,
      chatCitations: s.chatCitations,
      selectNode: s.selectNode,
    })),
  )

  const [input, setInput] = useState('')

  if (!showChat) return null

  const handleSend = () => {
    if (!input.trim() || !selectedVaultId) return
    const msg = input.trim()
    setInput('')
    sendChatMessage(msg)
  }

  return (
    <div className="w-80 border-l bg-card flex flex-col shrink-0 h-full">
      <div className="p-3 border-b flex items-center justify-between">
        <h2 className="font-semibold text-sm">Chat with Graph</h2>
        <button className="text-xs text-muted-foreground hover:text-foreground" onClick={toggleChat}>×</button>
      </div>

      <div className="flex-1 overflow-y-auto p-3 space-y-3">
        {chatMessages.map((msg: ChatMessage, i: number) => (
          <div key={i} className={`text-sm ${msg.role === 'user' ? 'text-right' : ''}`}>
            <div className={`inline-block px-3 py-1.5 rounded-lg max-w-[90%] whitespace-pre-wrap ${msg.role === 'user' ? 'bg-primary text-primary-foreground' : 'bg-muted'}`}>
              {msg.content}
              {msg.role === 'assistant' && chatLoading && i === chatMessages.length - 1 && !msg.content && (
                <span className="italic text-muted-foreground">Thinking...</span>
              )}
            </div>
          </div>
        ))}

        {chatCitations.length > 0 && (
          <div className="border-t pt-2 mt-2">
            <p className="text-[10px] text-muted-foreground uppercase tracking-wider mb-1">Sources</p>
            {chatCitations.map((c: Citation, i: number) => (
              <button
                key={i}
                className="w-full text-left px-2 py-1 text-xs rounded hover:bg-muted"
                onClick={() => selectNode(c.node_id)}
              >
                <span className="font-medium">{c.title}</span>
                <span className="text-muted-foreground ml-1">— {c.snippet.slice(0, 60)}...</span>
              </button>
            ))}
          </div>
        )}

      </div>

      <div className="p-3 border-t flex gap-2">
        <input
          className="flex-1 px-2 py-1 text-sm border rounded bg-background"
          placeholder="Ask about your knowledge graph..."
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === 'Enter') handleSend()
          }}
        />
        <button
          className="px-3 py-1 text-xs bg-primary text-primary-foreground rounded hover:opacity-90"
          onClick={handleSend}
          disabled={chatLoading}
        >
          Send
        </button>
      </div>
    </div>
  )
}
