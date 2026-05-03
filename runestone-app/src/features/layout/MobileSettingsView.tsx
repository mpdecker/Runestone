import { useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { useStore } from '@/store'
import { Server, Moon, Sun, Monitor } from 'lucide-react'

export function MobileSettingsView() {
  const { darkMode, toggleDarkMode } = useStore()
  const [serverUrl, setServerUrl] = useState(
    localStorage.getItem('runestone_server_url') || ''
  )
  const [connectionStatus, setConnectionStatus] = useState<string>('')

  const handleTestConnection = async () => {
    try {
      const result = await invoke<boolean>('test_connection')
      setConnectionStatus(result ? 'Connected' : 'Failed')
    } catch (e) {
      setConnectionStatus(String(e))
    }
  }

  const handleSaveServer = () => {
    localStorage.setItem('runestone_server_url', serverUrl.trim())
    invoke('configure_server_connection', {
      apiUrl: serverUrl.trim(),
      authToken: localStorage.getItem('runestone_auth_token') || null,
    })
      .then(() => setConnectionStatus('Saved'))
      .catch((e) => setConnectionStatus(String(e)))
  }

  return (
    <div className="h-full overflow-y-auto">
      <div className="p-4 border-b">
        <h2 className="text-lg font-semibold">Settings</h2>
      </div>

      <div className="p-4 space-y-6">
        <section>
          <h3 className="text-sm font-medium mb-3 flex items-center gap-2">
            <Server className="w-4 h-4" /> Server Connection
          </h3>
          <div className="space-y-2">
            <input
              type="text"
              value={serverUrl}
              onChange={(e) => setServerUrl(e.target.value)}
              placeholder="https://my-server.com"
              className="w-full px-3 py-2.5 min-h-[44px] rounded-lg border bg-background text-sm focus:outline-none focus:ring-2 focus:ring-primary/20"
            />
            <div className="flex gap-2">
              <button
                onClick={handleSaveServer}
                className="flex-1 py-2.5 min-h-[44px] rounded-lg bg-primary text-primary-foreground text-sm font-medium"
              >
                Save
              </button>
              <button
                onClick={handleTestConnection}
                className="flex-1 py-2.5 min-h-[44px] rounded-lg border text-sm font-medium"
              >
                Test
              </button>
            </div>
            {connectionStatus && (
              <p className={`text-xs ${connectionStatus === 'Connected' || connectionStatus === 'Saved' ? 'text-green-600' : 'text-destructive'}`}>
                {connectionStatus}
              </p>
            )}
          </div>
        </section>

        <section>
          <h3 className="text-sm font-medium mb-3 flex items-center gap-2">
            <Monitor className="w-4 h-4" /> Appearance
          </h3>
          <button
            onClick={toggleDarkMode}
            className="w-full flex items-center justify-between p-3 rounded-lg border min-h-[44px]"
          >
            <span className="text-sm">Dark Mode</span>
            <span className="flex items-center gap-2 text-sm text-muted-foreground">
              {darkMode ? (
                <Moon className="w-4 h-4" />
              ) : (
                <Sun className="w-4 h-4" />
              )}
              {darkMode ? 'On' : 'Off'}
            </span>
          </button>
        </section>

        <section>
          <h3 className="text-sm font-medium mb-3">About</h3>
          <div className="p-3 rounded-lg border">
            <p className="text-sm font-medium">Runestone</p>
            <p className="text-xs text-muted-foreground mt-1">Personal Knowledge Graph</p>
            <p className="text-xs text-muted-foreground">Version 0.1.0</p>
          </div>
        </section>
      </div>
    </div>
  )
}
