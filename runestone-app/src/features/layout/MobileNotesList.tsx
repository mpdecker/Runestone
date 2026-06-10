import { useState, useEffect } from 'react'
import { useStore } from '@/store'
import { NoteEditor } from '@/features/editor'
import { FileText, Plus, Search, X } from 'lucide-react'

export function MobileNotesList() {
  const { nodes, loadVaults, selectNode, selectedNodeId, createNode } = useStore()
  const [search, setSearch] = useState('')
  const [showEditor, setShowEditor] = useState(false)

  useEffect(() => {
    loadVaults()
  }, [loadVaults])

  const filtered = nodes.filter((n) =>
    n.title.toLowerCase().includes(search.toLowerCase())
  )

  if (showEditor && selectedNodeId) {
    return (
      <div className="h-full flex flex-col">
        <div className="flex items-center gap-2 p-2 border-b shrink-0">
          <button
            onClick={() => setShowEditor(false)}
            className="p-2 min-h-[44px] min-w-[44px] flex items-center justify-center rounded-lg hover:bg-muted"
          >
            <X className="w-5 h-5" />
          </button>
          <span className="font-medium text-sm truncate">Editor</span>
        </div>
        <div className="flex-1 min-h-0">
          <NoteEditor />
        </div>
      </div>
    )
  }

  return (
    <div className="h-full flex flex-col relative">
      <div className="p-3 border-b shrink-0">
        <div className="relative">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
          <input
            type="text"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder="Search notes..."
            className="w-full pl-9 pr-3 py-2.5 min-h-[44px] rounded-lg border bg-background text-sm focus:outline-none focus:ring-2 focus:ring-primary/20"
          />
        </div>
      </div>

      <div className="flex-1 overflow-y-auto">
        {filtered.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-full text-muted-foreground gap-2 p-6">
            <FileText className="w-8 h-8" />
            <p className="text-sm">No notes yet</p>
          </div>
        ) : (
          filtered.map((node) => (
            <button
              key={node.id}
              onClick={() => {
                selectNode(node.id)
                setShowEditor(true)
              }}
              className="w-full text-left p-3 border-b hover:bg-muted/50 transition-colors min-h-[44px] flex items-center gap-3"
            >
              <FileText className="w-4 h-4 text-muted-foreground shrink-0" />
              <div className="min-w-0 flex-1">
                <p className="text-sm font-medium truncate">{node.title}</p>
                {node.content_type && (
                  <p className="text-xs text-muted-foreground">{node.content_type}</p>
                )}
              </div>
            </button>
          ))
        )}
      </div>

      <button
        onClick={async () => {
          const title = `Note ${new Date().toLocaleDateString()}`
          await createNode(title)
          setShowEditor(true)
        }}
        className="absolute bottom-20 right-4 w-14 h-14 rounded-full bg-primary text-primary-foreground shadow-lg flex items-center justify-center"
        aria-label="Create note"
      >
        <Plus className="w-6 h-6" />
      </button>
    </div>
  )
}
