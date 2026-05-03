import { useState, useEffect } from 'react'
import { useStore } from '@/store'
import * as api from '@/lib/api'

export function ClipperPanel() {
  const { selectedVaultId } = useStore()
  const [port, setPort] = useState<number | null>(null)
  const [loading, setLoading] = useState(false)

  useEffect(() => {
    api.getClipperStatus().then((p) => {
      if (p) setPort(p)
    }).catch(() => {})
  }, [])

  if (!selectedVaultId) return null

  const start = async () => {
    setLoading(true)
    try {
      const p = await api.startClipperServer(selectedVaultId)
      setPort(p)
    } catch (e) {
      alert(String(e))
    }
    setLoading(false)
  }

  const stop = async () => {
    try {
      await api.stopClipperServer()
      setPort(null)
    } catch (e) {
      alert(String(e))
    }
  }

  return (
    <div className="border-t p-2 space-y-1">
      <p className="text-[10px] text-muted-foreground uppercase tracking-wider">Web Clipper</p>
      {port ? (
        <div className="space-y-1">
          <p className="text-[10px] text-muted-foreground">
            Server running on port <span className="text-accent-foreground">{port}</span>
          </p>
          <p className="text-[10px] text-muted-foreground">
            Install the Chrome extension and set port to {port}.
          </p>
          <button
            className="w-full text-[10px] px-2 py-1 rounded border border-border hover:bg-muted text-muted-foreground"
            onClick={stop}
          >
            Stop Clipper
          </button>
        </div>
      ) : (
        <button
          className="w-full text-[10px] px-2 py-1 rounded border border-border hover:bg-muted text-muted-foreground"
          onClick={start}
          disabled={loading}
        >
          {loading ? 'Starting...' : 'Start Web Clipper'}
        </button>
      )}
    </div>
  )
}
