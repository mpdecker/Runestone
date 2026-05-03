import { useState } from 'react'
import { useStore } from '@/store'
import { Button } from '@/components/ui/button'
import { VaultList } from '@/features/vault/VaultList'
import { FilterBar } from './FilterBar'
import { NodeList } from './NodeList'
import { NodeActions } from './NodeActions'
import { ObsidianImport } from './ObsidianImport'
import { TagPane } from './TagPane'
import { PropertiesPanel } from './PropertiesPanel'
import { OutlinePanel } from './OutlinePanel'
import { VersionsPanel } from './VersionsPanel'
import { CssSnippets } from './CssSnippets'
import { ClipperPanel } from './ClipperPanel'
import { PluginPanel } from './PluginPanel'

import { FileTree } from './FileTree'

export function Sidebar() {
  const {
    selectedVaultId,
    scanVault, createNode, isLoading, error,
    toggleSearch, showSearch, selectNode,
    toggleExtractions, showExtractions, pendingExtractions,
    toggleChat, showChat,
    sidebarCollapsed, toggleSidebar,
    listViewMode, setListViewMode,
    registeredPanels,
  } = useStore()

  const [showObsidianImport, setShowObsidianImport] = useState(false)
  const [showNewNode, setShowNewNode] = useState(false)
  const [newTitle, setNewTitle] = useState('')

  return (
    <div className={`h-full border-r bg-card flex flex-col transition-all duration-200 ${sidebarCollapsed ? 'w-12' : 'w-72'}`}>
      {sidebarCollapsed ? (
        <div className="flex flex-col items-center pt-3 gap-2">
          <Button variant="ghost" size="icon" className="h-8 w-8" onClick={toggleSidebar} title="Expand sidebar (Ctrl+Shift+B)">
            <span className="text-sm">\u25B6</span>
          </Button>
          {selectedVaultId && (
            <>
              <Button variant="ghost" size="icon" className="h-7 w-7" onClick={toggleSearch} title="Search (Ctrl+K)">
                <span className="text-xs">Q</span>
              </Button>
              <Button variant="ghost" size="icon" className="h-7 w-7" onClick={toggleChat} title="Chat (Ctrl+L)">
                <span className="text-xs">C</span>
              </Button>
              <Button variant="ghost" size="icon" className="h-7 w-7" onClick={() => { toggleSidebar(); createNode('Untitled') }} title="New Note (Ctrl+N)">
                <span className="text-xs">+</span>
              </Button>
            </>
          )}
        </div>
      ) : (
        <>
          <VaultList />

          {selectedVaultId && (
            <div className="flex-1 flex flex-col min-h-0">
              <div className="p-3 border-b flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <h2 className="font-semibold text-sm">Notes</h2>
                  <button
                    className={`text-[10px] px-1.5 py-0.5 rounded border ${listViewMode === 'list' ? 'bg-accent border-accent text-accent-foreground' : 'text-muted-foreground border-border hover:bg-muted'}`}
                    onClick={() => setListViewMode('list')}
                    title="List view"
                  >
                    List
                  </button>
                  <button
                    className={`text-[10px] px-1.5 py-0.5 rounded border ${listViewMode === 'tree' ? 'bg-accent border-accent text-accent-foreground' : 'text-muted-foreground border-border hover:bg-muted'}`}
                    onClick={() => setListViewMode('tree')}
                    title="Tree view"
                  >
                    Tree
                  </button>
                </div>
                <div className="flex gap-1">
                  <Button variant="ghost" size="icon" className={`h-7 w-7 ${showSearch ? 'bg-accent' : ''}`} onClick={toggleSearch} title="Search">
                    <span className="text-xs">Q</span>
                  </Button>
                  <Button variant="ghost" size="icon" className={`h-7 w-7 ${showChat ? 'bg-accent' : ''}`} onClick={toggleChat} title="Chat (Ctrl+L)">
                    <span className="text-xs">C</span>
                  </Button>
                  <Button variant="ghost" size="icon" className={`h-7 w-7 ${showExtractions ? 'bg-accent' : ''}`} onClick={toggleExtractions} title="Extractions">
                    <span className="text-xs relative">E{pendingExtractions.length > 0 && <span className="absolute -top-0.5 -right-1 text-[8px] text-amber-400">{pendingExtractions.length}</span>}</span>
                  </Button>
                  <Button variant="ghost" size="icon" className="h-7 w-7" onClick={scanVault} title="Scan vault">
                    <span className="text-xs">S</span>
                  </Button>
                  <Button variant="ghost" size="icon" className="h-7 w-7" onClick={() => setShowObsidianImport(!showObsidianImport)} title="Import Obsidian">
                    <span className="text-xs">O</span>
                  </Button>
                  <Button variant="ghost" size="icon" className="h-7 w-7" onClick={() => setShowNewNode(!showNewNode)}>
                    <span className="text-lg leading-none">+</span>
                  </Button>
                  <Button
                    variant="ghost"
                    size="sm"
                    className="text-[10px] h-6 px-1"
                    onClick={async () => {
                      if (selectedVaultId) {
                        try {
                          const { createDailyNote } = await import('@/lib/api')
                          const node = await createDailyNote(selectedVaultId)
                          selectNode(node.id)
                        } catch (e) {
                          alert(`Daily note failed: ${e}`)
                        }
                      }
                    }}
                    title="Open today's daily note"
                  >
                    Today
                  </Button>
                  <Button
                    variant="ghost"
                    size="sm"
                    className="text-[10px] h-6 px-1"
                    onClick={async () => {
                      if (selectedVaultId) {
                        try {
                          const { getRandomNode } = await import('@/lib/api')
                          const node = await getRandomNode(selectedVaultId)
                          selectNode(node.id)
                        } catch {
                          // no nodes available
                        }
                      }
                    }}
                    title="Open a random note"
                  >
                    Random
                  </Button>
                </div>
              </div>

              <ObsidianImport />

              <FilterBar />

              {showNewNode && (
                <div className="p-2 border-b space-y-1">
                  <input
                    className="w-full px-2 py-1 text-sm border rounded bg-background"
                    placeholder="Note title"
                    value={newTitle}
                    onChange={(e) => setNewTitle(e.target.value)}
                    onKeyDown={async (e) => {
                      if (e.key === 'Enter' && newTitle) {
                        await createNode(newTitle)
                        setNewTitle('')
                        setShowNewNode(false)
                      }
                    }}
                    autoFocus
                  />
                </div>
              )}

              {listViewMode === 'list' ? <NodeList /> : <FileTree />}

              <NodeActions />

              <OutlinePanel />

              <TagPane />

              <PropertiesPanel />

              <VersionsPanel />

              <CssSnippets />

              <ClipperPanel />

              <PluginPanel />

              {registeredPanels?.map((panel) => (
                <div key={panel.id} className="border-t p-2" ref={(el) => {
                  if (el && !el.hasChildNodes()) {
                    panel.render(el)
                  }
                }} />
              ))}
            </div>
          )}

          {error && (
            <div className="p-2 border-t text-xs text-destructive bg-destructive/10">
              {error}
            </div>
          )}

          {isLoading && (
            <div className="p-2 border-t text-xs text-muted-foreground">
              Loading...
            </div>
          )}
        </>
      )}
    </div>
  )
}
