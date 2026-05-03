import { useState } from 'react'
import { useStore } from '@/store'
import { Button } from '@/components/ui/button'

export function VaultList() {
  const {
    vaults, selectedVaultId,
    createVault, selectVault,
    initDb, toggleDarkMode, darkMode,
    toggleSidebar,
  } = useStore()

  const [showNewVault, setShowNewVault] = useState(false)
  const [vaultName, setVaultName] = useState('')
  const [vaultPath, setVaultPath] = useState('')

  return (
    <div className="p-3 border-b">
      <div className="flex items-center justify-between mb-2">
        <h2 className="font-semibold text-sm">Vaults</h2>
        <div className="flex gap-1">
          <Button variant="ghost" size="icon" className="h-7 w-7" onClick={toggleDarkMode} title="Toggle dark mode">
            <span className="text-xs">{darkMode ? '\u2600' : '\u263E'}</span>
          </Button>
          <Button variant="ghost" size="icon" className="h-7 w-7" onClick={initDb} title="Init DB">
            <span className="text-xs">DB</span>
          </Button>
          <Button variant="ghost" size="icon" className="h-7 w-7" onClick={() => setShowNewVault(!showNewVault)}>
            <span className="text-lg leading-none">+</span>
          </Button>
          <Button variant="ghost" size="icon" className="h-7 w-7" onClick={toggleSidebar} title="Collapse sidebar (Ctrl+Shift+B)">
            <span className="text-sm">\u25C0</span>
          </Button>
        </div>
      </div>

      {showNewVault && (
        <div className="mb-2 space-y-1">
          <input
            className="w-full px-2 py-1 text-sm border rounded bg-background"
            placeholder="Vault name"
            value={vaultName}
            onChange={(e) => setVaultName(e.target.value)}
          />
          <input
            className="w-full px-2 py-1 text-sm border rounded bg-background"
            placeholder="Root path"
            value={vaultPath}
            onChange={(e) => setVaultPath(e.target.value)}
          />
          <div className="flex gap-1">
            <Button size="sm" variant="outline" className="text-xs h-7"
              onClick={async () => {
                if (vaultName && vaultPath) {
                  await createVault(vaultName, vaultPath)
                  setVaultName('')
                  setVaultPath('')
                  setShowNewVault(false)
                }
              }}
            >
              Create
            </Button>
            <Button size="sm" variant="ghost" className="text-xs h-7"
              onClick={() => setShowNewVault(false)}
            >
              Cancel
            </Button>
          </div>
        </div>
      )}

      <div className="space-y-0.5 max-h-32 overflow-y-auto">
        {vaults.map((v) => (
          <button
            key={v.id}
            className={`w-full text-left px-2 py-1 text-sm rounded ${selectedVaultId === v.id ? 'bg-accent text-accent-foreground' : 'hover:bg-muted'}`}
            onClick={() => selectVault(v.id)}
          >
            {v.name}
          </button>
        ))}
        {vaults.length === 0 && (
          <p className="text-xs text-muted-foreground px-2 py-1">No vaults yet</p>
        )}
      </div>
    </div>
  )
}
