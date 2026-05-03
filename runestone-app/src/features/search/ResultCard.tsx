import type { SearchResult } from '@/lib/types'

function ResultCard({ result, onClick }: { result: SearchResult; onClick: (id: string) => void }) {
  return (
    <button
      className="w-full text-left p-2 rounded hover:bg-muted border-b border-border/50"
      onClick={() => onClick(result.node_id)}
    >
      <div className="flex items-center gap-2 mb-1">
        <span className="font-medium text-sm">{result.title}</span>
        <span className="text-[10px] px-1 py-0.5 rounded bg-muted text-muted-foreground">
          {result.content_type}
        </span>
        <span className="text-[10px] text-muted-foreground ml-auto">
          {result.score.toFixed(3)}
        </span>
      </div>
      <p className="text-xs text-muted-foreground line-clamp-2">{result.snippet}</p>
    </button>
  )
}

export { ResultCard }
