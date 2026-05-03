import { useState } from 'react'
import type { Backlink, TagSuggestion } from '@/lib/types'
import { useStore } from '@/store'
import { Button } from '@/components/ui/button'

export function NodeActions() {
  const {
    selectedNodeId, currentNode,
    showBacklinks, toggleBacklinks, parseWikiLinks,
    showOutgoingLinks, toggleOutgoingLinks, outgoingLinks,
    summarizeNode, nodeSummary, summaryLoading,
    suggestedLinks, loadSuggestedLinks,
    suggestTags, tagSuggestions,
    backlinks, selectNode,
    nodeTags, addTags, removeTag, acceptSuggestedTags,
    selectedVaultId,
  } = useStore()

  const [tagInput, setTagInput] = useState('')

  if (!selectedNodeId) return null

  const handleAddTag = async (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && tagInput.trim()) {
      const tags = tagInput.split(',').map(t => t.trim()).filter(Boolean)
      await addTags(selectedNodeId, tags)
      setTagInput('')
    }
  }

  return (
    <div className="border-t p-2 space-y-1">
      <div className="flex gap-1 flex-wrap">
        <Button
          variant="ghost"
          size="sm"
          className="text-xs h-7"
          onClick={toggleBacklinks}
        >
          {showBacklinks ? 'Hide' : 'Show'} Backlinks
        </Button>
        <Button
          variant="ghost"
          size="sm"
          className="text-xs h-7"
          onClick={toggleOutgoingLinks}
        >
          {showOutgoingLinks ? 'Hide' : 'Show'} Outgoing
        </Button>
        {currentNode && (
          <Button
            variant="ghost"
            size="sm"
            className="text-xs h-7"
            onClick={() => parseWikiLinks(selectedNodeId)}
          >
            Parse Links
          </Button>
        )}
        <Button
          variant="ghost"
          size="sm"
          className="text-xs h-7"
          onClick={() => {
            summarizeNode(selectedNodeId)
            loadSuggestedLinks(selectedNodeId)
            suggestTags(selectedNodeId)
          }}
          disabled={summaryLoading}
        >
          {summaryLoading ? '...' : 'Summarize'}
        </Button>
        <Button
          variant="ghost"
          size="sm"
          className="text-xs h-7"
          onClick={async () => {
            if (selectedNodeId) {
              try {
                const { exportNodeToMarkdown } = await import('@/lib/api')
                const path = await exportNodeToMarkdown(selectedNodeId)
                alert(`Exported to: ${path}`)
              } catch (e) {
                alert(`Export failed: ${e}`)
              }
            }
          }}
        >
          Export MD
        </Button>
      </div>

      {nodeSummary && (
        <div className="p-2 bg-muted/50 rounded text-xs text-muted-foreground">
          {nodeSummary}
        </div>
      )}

      {tagSuggestions.length > 0 && (
        <div className="space-y-0.5">
          <div className="flex items-center justify-between">
            <p className="text-[10px] text-muted-foreground uppercase tracking-wider">Suggested Tags</p>
            <Button
              variant="ghost"
              size="sm"
              className="text-[10px] h-5 px-1"
              onClick={() => {
                const names = tagSuggestions.map((t: TagSuggestion) => t.name)
                acceptSuggestedTags(selectedNodeId, names)
              }}
            >
              Accept All
            </Button>
          </div>
          <div className="flex flex-wrap gap-1">
            {tagSuggestions.map((t: TagSuggestion, i: number) => (
              <button
                key={i}
                className="px-1.5 py-0.5 text-[10px] rounded bg-muted text-muted-foreground hover:bg-accent hover:text-accent-foreground"
                title={t.reason}
                onClick={() => addTags(selectedNodeId, [t.name])}
              >
                #{t.name} ({Math.round(t.confidence * 100)}%)
              </button>
            ))}
          </div>
        </div>
      )}

      {nodeTags && (
        <div className="space-y-0.5">
          <p className="text-[10px] text-muted-foreground uppercase tracking-wider">Tags</p>
          <div className="flex flex-wrap gap-1">
            {nodeTags.tags.length > 0 ? (
              nodeTags.tags.map((tag) => (
                <span
                  key={tag}
                  className="px-1.5 py-0.5 text-[10px] rounded bg-accent/50 text-accent-foreground flex items-center gap-1 group"
                >
                  #{tag}
                  <button
                    className="opacity-0 group-hover:opacity-100 hover:text-destructive"
                    onClick={() => removeTag(selectedNodeId, tag)}
                    title={`Remove tag: ${tag}`}
                  >
                    &times;
                  </button>
                </span>
              ))
            ) : (
              <span className="text-[10px] text-muted-foreground">None</span>
            )}
          </div>
          <input
            className="w-full px-2 py-1 text-[10px] border rounded bg-background"
            placeholder="Add tags (comma-separated)..."
            value={tagInput}
            onChange={(e) => setTagInput(e.target.value)}
            onKeyDown={handleAddTag}
          />
        </div>
      )}

      {suggestedLinks.length > 0 && (
        <div className="space-y-0.5">
          <p className="text-[10px] text-muted-foreground uppercase tracking-wider">Suggested Links</p>
          {suggestedLinks.map((s) => (
            <button
              key={s.node_id}
              className="w-full text-left px-2 py-0.5 text-xs rounded hover:bg-muted flex items-center justify-between"
              onClick={() => selectNode(s.node_id)}
            >
              <span className="truncate">{s.title}</span>
              <span className="text-[10px] text-muted-foreground shrink-0 ml-1">{Math.round(s.score * 100)}%</span>
            </button>
          ))}
        </div>
      )}

      {showBacklinks && backlinks.length > 0 && (
        <div className="space-y-0.5 max-h-32 overflow-y-auto">
          <p className="text-[10px] text-muted-foreground uppercase tracking-wider">Backlinks</p>
          {backlinks.map((b: Backlink) => (
            <button
              key={b.node_id}
              className="w-full text-left px-2 py-0.5 text-xs rounded hover:bg-muted"
              onClick={() => selectNode(b.node_id)}
            >
              <div className="truncate">{b.title}</div>
              {b.context && (
                <div className="text-[10px] text-muted-foreground truncate">{b.context}</div>
              )}
            </button>
          ))}
        </div>
      )}
      {showBacklinks && backlinks.length === 0 && (
        <p className="text-[10px] text-muted-foreground">No backlinks found</p>
      )}

      {showOutgoingLinks && outgoingLinks.length > 0 && (
        <div className="space-y-0.5 max-h-32 overflow-y-auto">
          <p className="text-[10px] text-muted-foreground uppercase tracking-wider">Outgoing Links</p>
          {outgoingLinks.map((b: Backlink) => (
            <button
              key={b.node_id}
              className="w-full text-left px-2 py-0.5 text-xs rounded hover:bg-muted"
              onClick={() => b.node_id !== '00000000-0000-0000-0000-000000000000' && selectNode(b.node_id)}
            >
              <div className="truncate">{b.title}</div>
              {b.context && (
                <div className="text-[10px] text-muted-foreground truncate">{b.context}</div>
              )}
            </button>
          ))}
        </div>
      )}
      {showOutgoingLinks && outgoingLinks.length === 0 && (
        <p className="text-[10px] text-muted-foreground">No outgoing links found</p>
      )}
    </div>
  )
}
