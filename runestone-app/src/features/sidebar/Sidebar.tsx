import { useState, useCallback } from 'react'
import { useShallow } from 'zustand/react/shallow'
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
import { PanelRightOpen, PanelRightClose } from 'lucide-react'

export function Sidebar() {
  const {
    selectedVaultId,
    scanVault, createNode, isLoading, error,
    nodeError, nodeLoading,
    graphError, graphLoading,
    searchError, searchLoading,
    clipperError,
    toggleSearch, showSearch, selectNode,
    toggleExtractions, showExtractions, pendingExtractions,
    toggleChat, showChat,
    sidebarCollapsed, toggleSidebar,
    listViewMode, setListViewMode,
    registeredPanels,
  } = useStore(
    useShallow((s) => ({
      selectedVaultId: s.selectedVaultId,
      scanVault: s.scanVault,
      createNode: s.createNode,
      isLoading: s.isLoading,
      error: s.error,
      nodeError: s.nodeError,
      nodeLoading: s.nodeLoading,
      graphError: s.graphError,
      graphLoading: s.graphLoading,
      searchError: s.searchError,
      searchLoading: s.searchLoading,
      clipperError: s.clipperError,
      toggleSearch: s.toggleSearch,
      showSearch: s.showSearch,
      selectNode: s.selectNode,
      toggleExtractions: s.toggleExtractions,
      showExtractions: s.showExtractions,
      pendingExtractions: s.pendingExtractions,
      toggleChat: s.toggleChat,
      showChat: s.showChat,
      sidebarCollapsed: s.sidebarCollapsed,
      toggleSidebar: s.toggleSidebar,
      listViewMode: s.listViewMode,
      setListViewMode: s.setListViewMode,
      registeredPanels: s.registeredPanels,
    })),
  )

  const [showObsidianImport, setShowObsidianImport] = useState(false)
  const [showNewNode, setShowNewNode] = useState(false)
  const [newTitle, setNewTitle] = useState('')
  const [hoverOpen, setHoverOpen] = useState(false)

  const handleMouseEnter = useCallback(() => {
    if (sidebarCollapsed) setHoverOpen(true)
  }, [sidebarCollapsed])

  const handleMouseLeave = useCallback(() => {
    setHoverOpen(false)
  }, [])

  const displayError = error || nodeError || graphError || searchError || clipperError
  const displayLoading = isLoading || nodeLoading || graphLoading || searchLoading

  return (
    <>
      {/* Hover trigger strip — always present when collapsed */}
      {sidebarCollapsed && !hoverOpen && (
        <div
          className="fixed left-0 top-0 bottom-0 w-2 z-40 cursor-col-resize hover:bg-primary/20 transition-colors"
          onMouseEnter={handleMouseEnter}
          title="Hover for sidebar"
        />
      )}

      {/* Sidebar panel */}
      <div
        className={`h-full bg-card flex flex-col border-r border-border transition-all duration-200 overflow-hidden
          ${sidebarCollapsed
            ? `fixed left-0 top-0 bottom-0 z-50 shadow-2xl ${hoverOpen ? 'w-72 opacity-100' : 'w-72 opacity-0 pointer-events-none -translate-x-2'}`
            : 'w-72 relative'}`}
        onMouseLeave={handleMouseLeave}
      >
        {/* Header row */}
        <div className="flex items-center justify-between px-3 py-2 border-b shrink-0">
          <h2 className="font-semibold text-sm">Runestone</h2>
          <Button
            variant="ghost"
            size="icon"
            className="h-7 w-7"
            onClick={toggleSidebar}
            title={sidebarCollapsed ? 'Pin sidebar open' : 'Collapse sidebar (Ctrl+Shift+B)'}
          >
            {sidebarCollapsed ? (
              <PanelRightOpen className="w-4 h-4" />
            ) : (
              <PanelRightClose className="w-4 h-4" />
            )}
          </Button>
        </div>

        <VaultList />

        {selectedVaultId && (
          <div className="flex-1 flex flex-col min-h-0">
            <div className="p-2 border-b flex items-center justify-between flex-wrap gap-1">
              <div className="flex items-center gap-1">
                <h3 className="font-medium text-xs">Notes</h3>
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
              <div className="flex gap-0.5 flex-wrap">
                <Button variant="ghost" size="icon" className={`h-6 w-6 ${showSearch ? 'bg-accent' : ''}`} onClick={toggleSearch} title="Search (Ctrl+K)" aria-label="Search">
                  <span className="text-[10px]">Q</span>
                </Button>
                <Button variant="ghost" size="icon" className={`h-6 w-6 ${showChat ? 'bg-accent' : ''}`} onClick={toggleChat} title="Chat (Ctrl+L)" aria-label="Chat">
                  <span className="text-[10px]">C</span>
                </Button>
                <Button variant="ghost" size="icon" className={`h-6 w-6 ${showExtractions ? 'bg-accent' : ''}`} onClick={toggleExtractions} title="Extractions" aria-label="Extractions">
                  <span className="text-[10px] relative">
                    E{pendingExtractions.length > 0 && <span className="absolute -top-0.5 -right-1 text-[7px] text-amber-400">{pendingExtractions.length}</span>}
                  </span>
                </Button>
                <Button variant="ghost" size="icon" className="h-6 w-6" onClick={scanVault} title="Scan vault" aria-label="Scan vault">
                  <span className="text-[10px]">S</span>
                </Button>
                <Button variant="ghost" size="icon" className="h-6 w-6" onClick={() => setShowObsidianImport(!showObsidianImport)} title="Import Obsidian vault" aria-label="Import Obsidian vault">
                  <span className="text-[10px]">O</span>
                </Button>
                <Button variant="ghost" size="icon" className="h-6 w-6" onClick={() => setShowNewNode(!showNewNode)} aria-label="Create note">
                  <span className="text-sm leading-none">+</span>
                </Button>
                <Button
                  variant="ghost"
                  size="sm"
                  className="text-[10px] h-5 px-1"
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
                  className="text-[10px] h-5 px-1"
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

            <ObsidianImport show={showObsidianImport} onClose={() => setShowObsidianImport(false)} />

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

        {displayError && (
          <div className="p-2 border-t text-xs text-destructive bg-destructive/10 shrink-0">
            {displayError}
          </div>
        )}

        {displayLoading && (
          <div className="p-2 border-t text-xs text-muted-foreground shrink-0">
            Loading...
          </div>
        )}
      </div>
    </>
  )
}
