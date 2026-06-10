import { useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { Link, Server } from 'lucide-react'

interface ConnectionScreenProps {
  onConnected: () => void
}

export function ConnectionScreen({ onConnected }: ConnectionScreenProps) {
  const [url, setUrl] = useState('')
  const [token, setToken] = useState('')
  const [status, setStatus] = useState<'idle' | 'connecting' | 'error'>('idle')
  const [error, setError] = useState('')

  const handleConnect = async () => {
    if (!url.trim()) return
    setStatus('connecting')
    setError('')

    try {
      await invoke('configure_server_connection', {
        apiUrl: url.trim(),
        authToken: token || null,
      })
      const ok = await invoke<boolean>('test_connection')
      if (!ok) {
        throw new Error('Server health check failed')
      }
      localStorage.setItem('runestone_server_url', url.trim())
      if (token) {
        localStorage.setItem('runestone_auth_token', token)
      }
      onConnected()
    } catch (e) {
      setStatus('error')
      setError(String(e))
    }
  }

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') handleConnect()
  }

  return (
    <div className="h-screen flex flex-col items-center justify-center p-6 bg-background text-foreground">
      <div className="w-full max-w-sm flex flex-col items-center gap-6">
        <div className="flex flex-col items-center gap-2">
          <div className="w-16 h-16 rounded-2xl bg-primary/10 flex items-center justify-center">
            <Server className="w-8 h-8 text-primary" />
          </div>
          <h1 className="text-xl font-semibold">Connect to Runestone</h1>
          <p className="text-sm text-muted-foreground text-center">
            Enter the URL of your Runestone server to get started. Remote mode is preview-only until a server is deployed — see docs/remote-mode.md.
          </p>
        </div>

        <div className="w-full flex flex-col gap-3">
          <div>
            <label className="block text-sm font-medium mb-1" htmlFor="server-url">
              Server URL
            </label>
            <div className="relative">
              <Link className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
              <input
                id="server-url"
                type="text"
                value={url}
                onChange={(e) => setUrl(e.target.value)}
                onKeyDown={handleKeyDown}
                placeholder="https://my-server.com"
                className="w-full pl-9 pr-3 py-2.5 min-h-[44px] rounded-lg border bg-background text-sm focus:outline-none focus:ring-2 focus:ring-primary/20"
                autoFocus
              />
            </div>
          </div>

          <div>
            <label className="block text-sm font-medium mb-1" htmlFor="auth-token">
              Auth Token <span className="text-muted-foreground font-normal">(optional)</span>
            </label>
            <input
              id="auth-token"
              type="password"
              value={token}
              onChange={(e) => setToken(e.target.value)}
              onKeyDown={handleKeyDown}
              placeholder="Bearer token"
              className="w-full px-3 py-2.5 min-h-[44px] rounded-lg border bg-background text-sm focus:outline-none focus:ring-2 focus:ring-primary/20"
            />
          </div>

          {status === 'error' && (
            <p className="text-sm text-destructive">{error}</p>
          )}

          <button
            onClick={handleConnect}
            disabled={status === 'connecting' || !url.trim()}
            className="w-full py-2.5 min-h-[44px] rounded-lg bg-primary text-primary-foreground font-medium text-sm disabled:opacity-50 hover:opacity-90 transition-opacity"
          >
            {status === 'connecting' ? 'Connecting...' : 'Connect'}
          </button>
        </div>
      </div>
    </div>
  )
}
