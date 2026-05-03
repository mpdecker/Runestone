import { useState, useEffect } from 'react'
import { MobileGraphView } from '@/features/layout/MobileGraphView'
import { MobileNotesList } from '@/features/layout/MobileNotesList'
import { MobileSearchView } from '@/features/layout/MobileSearchView'
import { MobileSettingsView } from '@/features/layout/MobileSettingsView'
import { MobileTabBar } from '@/features/layout/MobileTabBar'
import { ConnectionScreen } from '@/features/layout/ConnectionScreen'
import { useStore } from '@/store'

type Tab = 'graph' | 'notes' | 'search' | 'settings'

export function MobileApp() {
  const [activeTab, setActiveTab] = useState<Tab>('notes')
  const [connected, setConnected] = useState(false)
  const { loadVaults, initDb } = useStore()

  const hasServerUrl = () => !!localStorage.getItem('runestone_server_url')

  useEffect(() => {
    if (hasServerUrl()) {
      setConnected(true)
    }
  }, [])

  useEffect(() => {
    if (connected) {
      initDb().then(() => loadVaults())
    }
  }, [connected, initDb, loadVaults])

  if (!connected) {
    return <ConnectionScreen onConnected={() => setConnected(true)} />
  }

  return (
    <div className="h-screen flex flex-col bg-background text-foreground overflow-hidden">
      <div className="flex-1 min-h-0">
        {activeTab === 'graph' && <MobileGraphView />}
        {activeTab === 'notes' && <MobileNotesList />}
        {activeTab === 'search' && <MobileSearchView />}
        {activeTab === 'settings' && <MobileSettingsView />}
      </div>
      <MobileTabBar activeTab={activeTab} onTabChange={setActiveTab} />
    </div>
  )
}
