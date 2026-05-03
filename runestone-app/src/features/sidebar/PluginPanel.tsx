import { useState } from 'react'
import { useStore } from '@/store'

export function PluginPanel() {
  const {
    installedPlugins, availablePlugins, pluginDir, pluginLoading,
    setPluginDir, discoverPlugins, loadPlugin, togglePlugin,
  } = useStore()
  const [dirInput, setDirInput] = useState(pluginDir || '')

  return (
    <div className="border-t p-2 space-y-1">
      <p className="text-[10px] text-muted-foreground uppercase tracking-wider">Plugins</p>

      <div className="flex gap-1">
        <input
          className="flex-1 px-1 py-0.5 text-[10px] border rounded bg-background min-w-0"
          placeholder="Plugins directory path..."
          value={dirInput}
          onChange={(e) => setDirInput(e.target.value)}
        />
        <button
          className="text-[10px] px-2 py-0.5 rounded border border-border hover:bg-muted shrink-0"
          onClick={() => {
            setPluginDir(dirInput)
            discoverPlugins()
          }}
        >
          Scan
        </button>
      </div>

      {pluginLoading && (
        <p className="text-[10px] text-muted-foreground">Discovering...</p>
      )}

      {availablePlugins.length > 0 && (
        <div className="space-y-0.5 max-h-32 overflow-y-auto">
          <p className="text-[10px] text-muted-foreground">Available ({availablePlugins.length})</p>
          {availablePlugins.map((p) => {
            const installed = installedPlugins.find((i) => i.manifest.name === p.name)
            return (
              <div key={p.name} className="flex items-center gap-1 text-[10px] p-1 rounded hover:bg-muted">
                <span className="flex-1 truncate">{p.name} <span className="text-muted-foreground">v{p.version}</span></span>
                {installed?.error && <span className="text-destructive text-[9px]">Error</span>}
                {!installed ? (
                  <button
                    className="text-[9px] px-1 py-0.5 rounded border border-border hover:bg-accent"
                    onClick={() => loadPlugin(p)}
                  >
                    Load
                  </button>
                ) : (
                  <button
                    className={`text-[9px] px-1 py-0.5 rounded border ${installed.enabled ? 'bg-accent border-accent text-accent-foreground' : 'border-border hover:bg-accent'}`}
                    onClick={() => togglePlugin(p.name)}
                  >
                    {installed.enabled ? 'On' : 'Off'}
                  </button>
                )}
              </div>
            )
          })}
        </div>
      )}

      {availablePlugins.length === 0 && pluginDir && !pluginLoading && (
        <p className="text-[10px] text-muted-foreground">No plugins found</p>
      )}
    </div>
  )
}
