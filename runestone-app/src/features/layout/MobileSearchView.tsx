import { useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { useStore } from '@/store'
import { Search, Loader2, FileText } from 'lucide-react'

interface SearchResult {
  node_id: string
  title: string
  content_type: string
  snippet: string
  score: number
}

export function MobileSearchView() {
  const { selectedVaultId } = useStore()
  const [query, setQuery] = useState('')
  const [results, setResults] = useState<SearchResult[]>([])
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState('')

  const handleSearch = async () => {
    if (!query.trim() || !selectedVaultId) return
    setLoading(true)
    setError('')
    try {
      const res = await invoke<SearchResult[]>('semantic_search', {
        query: { vault_id: selectedVaultId, query: query.trim(), limit: 20 },
      })
      setResults(res || [])
    } catch (e) {
      setError(String(e))
    }
    setLoading(false)
  }

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') handleSearch()
  }

  return (
    <div className="h-full flex flex-col">
      <div className="p-3 border-b shrink-0">
        <div className="relative">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
          <input
            type="text"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="Search your knowledge graph..."
            className="w-full pl-9 pr-12 py-2.5 min-h-[44px] rounded-lg border bg-background text-sm focus:outline-none focus:ring-2 focus:ring-primary/20"
          />
          <button
            onClick={handleSearch}
            disabled={loading || !query.trim() || !selectedVaultId}
            className="absolute right-1.5 top-1/2 -translate-y-1/2 p-1.5 min-h-[32px] min-w-[32px] rounded-md bg-primary text-primary-foreground disabled:opacity-50"
          >
            {loading ? (
              <Loader2 className="w-4 h-4 animate-spin" />
            ) : (
              <Search className="w-4 h-4" />
            )}
          </button>
        </div>
      </div>

      <div className="flex-1 overflow-y-auto">
        {error && (
          <div className="p-4 text-sm text-destructive">{error}</div>
        )}
        {!selectedVaultId && (
          <div className="p-4 text-sm text-muted-foreground">Select a vault to search.</div>
        )}
        {results.length === 0 && !loading && query && !error && selectedVaultId && (
          <div className="flex flex-col items-center justify-center h-32 text-muted-foreground gap-2">
            <FileText className="w-8 h-8" />
            <p className="text-sm">No results found</p>
          </div>
        )}
        {results.map((r) => (
          <div key={r.node_id} className="p-3 border-b">
            <div className="flex items-center gap-2 mb-1">
              <FileText className="w-4 h-4 text-muted-foreground shrink-0" />
              <p className="text-sm font-medium">{r.title}</p>
              <span className="text-xs text-muted-foreground ml-auto">
                {Math.round(r.score * 100)}%
              </span>
            </div>
            <p className="text-xs text-muted-foreground line-clamp-2">{r.snippet}</p>
          </div>
        ))}
        {!query && (
          <div className="flex flex-col items-center justify-center h-full text-muted-foreground gap-2 p-6">
            <Search className="w-8 h-8" />
            <p className="text-sm">Search across your notes</p>
          </div>
        )}
      </div>
    </div>
  )
}
