import { useStore } from '@/store'

const NODE_TYPES = ['note', 'concept', 'entity', 'document']

export function FilterBar() {
  const { filterText, filterTypes, setFilterText, toggleFilterType } = useStore()

  return (
    <div className="p-2 border-b space-y-1.5">
      <input
        className="w-full px-2 py-1 text-xs border rounded bg-background"
        placeholder="Search notes..."
        value={filterText}
        onChange={(e) => setFilterText(e.target.value)}
      />
      <div className="flex flex-wrap gap-1">
        {NODE_TYPES.map((t) => (
          <button
            key={t}
            className={`px-1.5 py-0.5 text-[10px] rounded border ${filterTypes.includes(t) ? 'bg-accent border-accent text-accent-foreground' : 'text-muted-foreground border-border hover:bg-muted'}`}
            onClick={() => toggleFilterType(t)}
          >
            {t}
          </button>
        ))}
      </div>
    </div>
  )
}
