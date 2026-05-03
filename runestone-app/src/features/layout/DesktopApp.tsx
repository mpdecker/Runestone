import { useEffect } from 'react'
import { Sidebar } from '@/features/sidebar'
import { NoteEditor } from '@/features/editor'
import { GraphCanvas } from '@/features/graph'
import { SearchPanel } from '@/features/search'
import { ExtractionReview } from '@/features/extraction'
import { CommandPalette } from '@/features/command-palette'
import { ChatPanel } from '@/features/chat'
import { TabBar } from '@/features/editor/TabBar'
import { useStore } from '@/store'

export function DesktopApp() {
  const {
    loadVaults, initDb, selectedNodeId,
    saveNode, createNode, toggleSearch,
    toggleExtractions, toggleCommandPalette,
    toggleChat, toggleSidebar, openTabs,
    splitMode, toggleSplitMode, secondaryTabId,
    secondaryNode, selectSecondaryNode, nodes,
    userCss, activeTabId, closeTab, closeAllTabs,
    switchToTab, deleteNode, toggleReadingMode,
  } = useStore()

  useEffect(() => {
    initDb().then(() => loadVaults())
  }, [])

  useEffect(() => {
    const existing = document.getElementById('runestone-user-css')
    if (existing) {
      if (userCss) {
        existing.textContent = userCss
      } else {
        existing.textContent = ''
      }
    } else if (userCss) {
      const style = document.createElement('style')
      style.id = 'runestone-user-css'
      style.textContent = userCss
      document.head.appendChild(style)
    }
  }, [userCss])

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      const mod = e.metaKey || e.ctrlKey

      if (mod && e.key === 's') {
        e.preventDefault()
        saveNode()
      } else if (mod && e.key === 'n') {
        e.preventDefault()
        createNode('Untitled')
      } else if (mod && e.key === 'k') {
        e.preventDefault()
        toggleSearch()
      } else if (mod && e.key === 'o') {
        e.preventDefault()
        toggleCommandPalette()
      } else if (mod && e.shiftKey && e.key === 'E') {
        e.preventDefault()
        toggleExtractions()
      } else if (mod && e.key === 'p') {
        e.preventDefault()
        toggleCommandPalette()
      } else if (mod && e.key === 'l') {
        e.preventDefault()
        toggleChat()
      } else if (mod && e.shiftKey && e.key === 'B') {
        e.preventDefault()
        toggleSidebar()
      } else if (mod && e.key === 'w') {
        e.preventDefault()
        if (e.shiftKey) {
          closeAllTabs()
        } else if (activeTabId) {
          closeTab(activeTabId)
        }
      } else if (mod && e.key === 'e') {
        e.preventDefault()
        toggleReadingMode()
      } else if (mod && e.shiftKey && e.key === 'D') {
        e.preventDefault()
        if (selectedNodeId) {
          if (window.confirm('Delete this note?')) {
            deleteNode(selectedNodeId)
          }
        }
      } else if (mod && e.key === 'Tab') {
        e.preventDefault()
        if (e.shiftKey) {
          const idx = openTabs.findIndex((t) => t.id === activeTabId)
          if (idx > 0) switchToTab(openTabs[idx - 1].id)
        } else {
          const idx = openTabs.findIndex((t) => t.id === activeTabId)
          if (idx < openTabs.length - 1) switchToTab(openTabs[idx + 1].id)
        }
      }
    }

    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [saveNode, createNode, toggleSearch, toggleExtractions, toggleCommandPalette, toggleChat, toggleSidebar, activeTabId, openTabs, closeTab, closeAllTabs, switchToTab, selectedNodeId, deleteNode, toggleReadingMode])

  return (
    <div className="h-screen flex bg-background text-foreground overflow-hidden">
      <Sidebar />
      <div className="flex-1 flex min-w-0">
        <GraphCanvas />
        {selectedNodeId && (
          <div className="w-96 border-l shrink-0 flex flex-col min-h-0">
            {openTabs.length > 0 && (
              <div className="flex items-center border-b bg-card shrink-0">
                <div className="flex-1 min-w-0">
                  <TabBar />
                </div>
                <button
                  className={`px-2 py-1 text-[10px] shrink-0 border-l hover:bg-muted ${splitMode === 'vertical' ? 'bg-accent text-accent-foreground' : 'text-muted-foreground'}`}
                  onClick={() => {
                    toggleSplitMode()
                    if (splitMode === 'off' && secondaryTabId) {
                      selectSecondaryNode(secondaryTabId)
                    }
                  }}
                  title="Toggle split pane"
                >
                  Split
                </button>
              </div>
            )}
            <div className={`flex-1 min-h-0 flex flex-col ${splitMode === 'vertical' ? 'divide-y' : ''}`}>
              <div className={splitMode === 'vertical' ? 'flex-1 min-h-0' : 'flex-1 min-h-0'}>
                <NoteEditor />
              </div>
              {splitMode === 'vertical' && secondaryTabId && secondaryNode && (
                <div className="flex-1 min-h-0">
                  <NoteEditor secondary />
                </div>
              )}
              {splitMode === 'vertical' && !secondaryTabId && (
                <div className="flex-1 flex items-center justify-center text-xs text-muted-foreground p-4">
                  Ctrl+click a note to open in split pane
                </div>
              )}
            </div>
          </div>
        )}
      </div>
      <SearchPanel />
      <ExtractionReview />
      <ChatPanel />
      <CommandPalette />
    </div>
  )
}
