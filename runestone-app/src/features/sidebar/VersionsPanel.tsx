import { useEffect, useState } from 'react'
import { useStore } from '@/store'
import type { NodeVersion } from '@/lib/types'
import * as api from '@/lib/api'

export function VersionsPanel() {
  const { selectedNodeId } = useStore()
  const [versions, setVersions] = useState<NodeVersion[]>([])
  const [loading, setLoading] = useState(false)

  useEffect(() => {
    if (!selectedNodeId) {
      setVersions([])
      return
    }
    setLoading(true)
    api.getNodeVersions(selectedNodeId)
      .then(setVersions)
      .catch(() => setVersions([]))
      .finally(() => setLoading(false))
  }, [selectedNodeId])

  if (!selectedNodeId) return null

  return (
    <div className="border-t p-2 space-y-1">
      <p className="text-[10px] text-muted-foreground uppercase tracking-wider">Version History</p>
      {loading ? (
        <p className="text-[10px] text-muted-foreground px-1">Loading...</p>
      ) : versions.length > 0 ? (
        <div className="max-h-32 overflow-y-auto space-y-0.5">
          {versions.map((v) => (
            <div key={v.id} className="flex items-center gap-1 group px-1 py-0.5 rounded hover:bg-muted">
              <span className="text-[10px] text-muted-foreground shrink-0 w-6">v{v.version_number}</span>
              <span className="text-[10px] text-muted-foreground shrink-0">
                {v.created_at ? new Date(v.created_at).toLocaleDateString() : 'N/A'}
              </span>
              <span className="text-[10px] text-muted-foreground">{v.word_count}w</span>
              <button
                className="text-[10px] ml-auto text-muted-foreground hover:text-accent-foreground opacity-0 group-hover:opacity-100"
                onClick={async () => {
                  try {
                    await api.restoreNodeVersion(v.id)
                    const { selectNode } = useStore.getState()
                    if (selectedNodeId) selectNode(selectedNodeId)
                  } catch (e) {
                    console.error('Restore failed:', e)
                  }
                }}
                title="Restore this version"
              >
                Restore
              </button>
            </div>
          ))}
        </div>
      ) : (
        <p className="text-[10px] text-muted-foreground px-1">No version history available</p>
      )}
    </div>
  )
}
