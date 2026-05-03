import { useStore } from '@/store'
import type { SearchResult } from '@/lib/types'
import { ResultCard } from './ResultCard'

export function SearchPanel() {
  const {
    showSearch, toggleSearch, searchQuery, setSearchQuery,
    runSearch, searchResults, searchLoading, similarNodes,
    selectNode, findSimilar, selectedNodeId,
  } = useStore()

  if (!showSearch) return null

  return (
    <div className="w-80 border-l bg-card flex flex-col shrink-0 h-full">
      <div className="p-3 border-b">
        <div className="flex items-center justify-between mb-2">
          <h2 className="font-semibold text-sm">Search</h2>
          <button
            className="text-xs text-muted-foreground hover:text-foreground"
            onClick={toggleSearch}
          >
            ×
          </button>
        </div>
        <div className="flex gap-1">
          <input
            className="flex-1 px-2 py-1 text-sm border rounded bg-background"
            placeholder="Search notes semantically..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === 'Enter') runSearch()
            }}
          />
          <button
            className="px-2 py-1 text-xs bg-primary text-primary-foreground rounded hover:opacity-90"
            onClick={runSearch}
            disabled={searchLoading}
          >
            Go
          </button>
        </div>
        {selectedNodeId && (
          <button
            className="mt-2 text-xs text-muted-foreground hover:text-foreground underline"
            onClick={() => findSimilar(selectedNodeId)}
          >
            Find similar to current note
          </button>
        )}
      </div>

      <div className="flex-1 overflow-y-auto">
        {searchLoading && (
          <p className="text-xs text-muted-foreground p-3">Searching...</p>
        )}

        {similarNodes.length > 0 && searchResults === null && (
          <div>
            <p className="text-[10px] text-muted-foreground uppercase tracking-wider px-3 pt-2 pb-1">
              Similar Notes ({similarNodes.length})
            </p>
            {similarNodes.map((r) => (
              <ResultCard key={r.node_id} result={r} onClick={selectNode} />
            ))}
          </div>
        )}

        {searchResults && (
          <div>
            <p className="text-[10px] text-muted-foreground uppercase tracking-wider px-3 pt-2 pb-1">
              Combined Results ({searchResults.combined.length})
            </p>
            {searchResults.combined.map((r: SearchResult) => (
              <ResultCard key={r.node_id} result={r} onClick={selectNode} />
            ))}
            {searchResults.combined.length === 0 && (
              <p className="text-xs text-muted-foreground p-3">No results found</p>
            )}
          </div>
        )}
      </div>
    </div>
  )
}
