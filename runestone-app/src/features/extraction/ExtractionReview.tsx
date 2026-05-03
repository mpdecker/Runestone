import { useState } from 'react'
import { useStore } from '@/store'
import type { PendingExtraction } from '@/lib/types'

export function ExtractionReview() {
  const {
    showExtractions, toggleExtractions,
    pendingExtractions, extractionResults,
    approveExtraction, rejectExtraction, batchApproveExtractions,
    selectedNodeId, extractFromDocument, isLoading,
  } = useStore()

  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set())

  const toggleSelect = (id: string) => {
    setSelectedIds((prev) => {
      const next = new Set(prev)
      if (next.has(id)) next.delete(id)
      else next.add(id)
      return next
    })
  }

  const selectAll = () => {
    setSelectedIds(new Set(pendingExtractions.map((p) => p.id)))
  }

  const clearSelection = () => {
    setSelectedIds(new Set())
  }

  if (!showExtractions) return null

  return (
    <div className="w-80 border-l bg-card flex flex-col shrink-0 h-full">
      <div className="p-3 border-b">
        <div className="flex items-center justify-between mb-2">
          <h2 className="font-semibold text-sm">Extractions</h2>
          <button
            className="text-xs text-muted-foreground hover:text-foreground"
            onClick={toggleExtractions}
          >
            ×
          </button>
        </div>

        {selectedNodeId && (
          <button
            className="w-full px-2 py-1 text-xs bg-primary text-primary-foreground rounded hover:opacity-90 mb-2"
            onClick={() => extractFromDocument(selectedNodeId)}
            disabled={isLoading}
          >
            {isLoading ? 'Extracting...' : 'Extract from selected document'}
          </button>
        )}

        {extractionResults.length > 0 && (
          <div className="mb-2 text-xs text-muted-foreground">
            Last extraction: {extractionResults.length} items found
          </div>
        )}
      </div>

      <div className="border-b px-3 py-1.5 flex items-center gap-2">
        <button className="text-[10px] text-muted-foreground hover:text-foreground" onClick={selectAll}>
          All
        </button>
        <button className="text-[10px] text-muted-foreground hover:text-foreground" onClick={clearSelection}>
          None
        </button>
        {selectedIds.size > 0 && (
          <button
            className="text-[10px] bg-primary text-primary-foreground px-1.5 py-0.5 rounded ml-auto"
            onClick={() => batchApproveExtractions([...selectedIds])}
          >
            Approve {selectedIds.size}
          </button>
        )}
      </div>

      <div className="flex-1 overflow-y-auto">
        {pendingExtractions.length === 0 && (
          <p className="text-xs text-muted-foreground p-3">
            No pending extractions. Import a document and run extraction.
          </p>
        )}
        {pendingExtractions.map((ext: PendingExtraction) => {
          const meta = ext.metadata
          return (
            <div
              key={ext.id}
              className={`border-b border-border/50 p-2 ${selectedIds.has(ext.id) ? 'bg-accent/50' : ''}`}
            >
              <div className="flex items-start gap-2">
                <input
                  type="checkbox"
                  checked={selectedIds.has(ext.id)}
                  onChange={() => toggleSelect(ext.id)}
                  className="mt-0.5"
                />
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-1 mb-1">
                    <span className="text-sm font-medium truncate">{ext.title}</span>
                    <span className="text-[10px] px-1 py-0.5 rounded bg-muted text-muted-foreground shrink-0">
                      {ext.content_type}
                    </span>
                    {meta && (
                      <span className="text-[10px] text-muted-foreground shrink-0">
                        {Math.round(meta.confidence * 100)}%
                      </span>
                    )}
                  </div>
                  {meta?.description && (
                    <p className="text-xs text-muted-foreground line-clamp-2">{meta.description}</p>
                  )}
                  <div className="flex gap-1 mt-1.5">
                    <button
                      className="text-[10px] px-1.5 py-0.5 bg-green-600 text-white rounded hover:opacity-90"
                      onClick={() => approveExtraction(ext.id)}
                    >
                      Approve
                    </button>
                    <button
                      className="text-[10px] px-1.5 py-0.5 bg-muted text-muted-foreground rounded hover:bg-destructive hover:text-destructive-foreground"
                      onClick={() => rejectExtraction(ext.id)}
                    >
                      Reject
                    </button>
                  </div>
                </div>
              </div>
            </div>
          )
        })}
      </div>
    </div>
  )
}
