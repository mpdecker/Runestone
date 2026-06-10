import { useState } from 'react'
import { useShallow } from 'zustand/react/shallow'
import { useStore } from '@/store'
import { Button } from '@/components/ui/button'

const NODES_PER_PAGE = 50

export function NodeList() {
  const { nodes, selectedNodeId, selectNode, deleteNode, filterText, filterTypes, splitMode, selectSecondaryNode, secondaryTabId } = useStore(
    useShallow((s) => ({
      nodes: s.nodes,
      selectedNodeId: s.selectedNodeId,
      selectNode: s.selectNode,
      deleteNode: s.deleteNode,
      filterText: s.filterText,
      filterTypes: s.filterTypes,
      splitMode: s.splitMode,
      selectSecondaryNode: s.selectSecondaryNode,
      secondaryTabId: s.secondaryTabId,
    })),
  )
  const [nodePage, setNodePage] = useState(0)

  const filteredNodes = nodes.filter((n) => {
    if (filterText && !n.title.toLowerCase().includes(filterText.toLowerCase())) return false
    if (filterTypes.length > 0 && !filterTypes.includes(n.content_type)) return false
    return true
  })

  const totalPages = Math.ceil(filteredNodes.length / NODES_PER_PAGE)
  const pagedNodes = filteredNodes.slice(nodePage * NODES_PER_PAGE, (nodePage + 1) * NODES_PER_PAGE)

  return (
    <div className="flex-1 overflow-y-auto p-2 space-y-0.5">
      {pagedNodes.map((n) => (
        <div
          key={n.id}
          className={`group flex items-center justify-between px-2 py-1 rounded text-sm cursor-pointer ${selectedNodeId === n.id || secondaryTabId === n.id ? 'bg-accent text-accent-foreground' : 'hover:bg-muted'}`}
          onClick={(e) => {
            if (splitMode === 'vertical' && (e.ctrlKey || e.metaKey)) {
              selectSecondaryNode(n.id)
            } else {
              selectNode(n.id)
            }
          }}
        >
          <span className="truncate flex-1">{n.title}</span>
          <Button
            variant="ghost"
            size="icon"
            className="h-5 w-5 opacity-0 group-hover:opacity-100"
            onClick={(e) => { e.stopPropagation(); deleteNode(n.id) }}
          >
            <span className="text-xs">x</span>
          </Button>
        </div>
      ))}
      {totalPages > 1 && (
        <div className="flex items-center justify-center gap-1 pt-1">
          <Button variant="ghost" size="sm" className="text-[10px] h-5 px-1" onClick={() => setNodePage(Math.max(0, nodePage - 1))} disabled={nodePage === 0}>\u2190</Button>
          <span className="text-[10px] text-muted-foreground">{nodePage + 1}/{totalPages}</span>
          <Button variant="ghost" size="sm" className="text-[10px] h-5 px-1" onClick={() => setNodePage(Math.min(totalPages - 1, nodePage + 1))} disabled={nodePage >= totalPages - 1}>\u2192</Button>
        </div>
      )}
      {filteredNodes.length === 0 && (
        <p className="text-xs text-muted-foreground px-2 py-1">No notes</p>
      )}
    </div>
  )
}
