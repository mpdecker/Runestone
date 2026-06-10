import { useShallow } from 'zustand/react/shallow'
import { useStore } from '@/store'
import { Button } from '@/components/ui/button'

export function TagPane() {
  const { vaultTags, selectedTag, selectTag, selectedVaultId } = useStore(
    useShallow((s) => ({
      vaultTags: s.vaultTags,
      selectedTag: s.selectedTag,
      selectTag: s.selectTag,
      selectedVaultId: s.selectedVaultId,
    })),
  )

  if (!selectedVaultId) return null

  return (
    <div className="border-t p-2 space-y-1">
      <div className="flex items-center justify-between">
        <p className="text-[10px] text-muted-foreground uppercase tracking-wider">Tags</p>
        {vaultTags.length > 0 && selectedTag && (
          <Button
            variant="ghost"
            size="sm"
            className="text-[10px] h-5 px-1"
            onClick={() => selectTag(null)}
          >
            Clear
          </Button>
        )}
      </div>
      {vaultTags.length > 0 ? (
        <div className="flex flex-wrap gap-1">
          {vaultTags.map((t) => (
            <button
              key={t.name}
              className={`px-1.5 py-0.5 text-[10px] rounded border transition-colors ${
                selectedTag === t.name
                  ? 'bg-accent border-accent text-accent-foreground'
                  : 'text-muted-foreground border-border hover:bg-muted'
              }`}
              onClick={() => selectTag(selectedTag === t.name ? null : t.name)}
              title={`${t.name} (${t.node_count ?? 0} notes)`}
            >
              #{t.name}
              {t.node_count != null && (
                <span className="ml-0.5 opacity-60">{t.node_count}</span>
              )}
            </button>
          ))}
        </div>
      ) : (
        <p className="text-[10px] text-muted-foreground px-1">No tags yet. Add tags to notes to organize them.</p>
      )}
    </div>
  )
}
