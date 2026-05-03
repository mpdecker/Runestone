import { useState, useMemo } from 'react'
import { useStore } from '@/store'

interface TreeNode {
  name: string
  path: string
  isFolder: boolean
  children: TreeNode[]
  nodeId?: string
  content_type?: string
}

export function buildTree(nodes: { id: string; title: string; content_type: string; file_path: string | null }[]): TreeNode[] {
  const root: TreeNode = { name: '/', path: '/', isFolder: true, children: [] }

  const itemsWithPath = nodes.filter((n) => n.file_path)
  const itemsWithoutPath = nodes.filter((n) => !n.file_path)

  for (const node of itemsWithPath) {
    const parts = (node.file_path! as string).replace(/\\/g, '/').split('/')
    const fileName = parts[parts.length - 1]
    const dirParts = parts.slice(0, -1)

    let current = root
    for (const dir of dirParts) {
      let child = current.children.find((c) => c.name === dir && c.isFolder)
      if (!child) {
        child = {
          name: dir,
          path: current.path === '/' ? `/${dir}` : `${current.path}/${dir}`,
          isFolder: true,
          children: [],
        }
        current.children.push(child)
      }
      current = child
    }

    current.children.push({
      name: node.title,
      path: node.file_path!,
      isFolder: false,
      children: [],
      nodeId: node.id,
      content_type: node.content_type,
    })
  }

  if (itemsWithoutPath.length > 0) {
    const unsortedFolder = root.children.find((c) => c.name === 'Unsorted' && c.isFolder)
    if (unsortedFolder) {
      for (const node of itemsWithoutPath) {
        unsortedFolder.children.push({
          name: node.title,
          path: node.title,
          isFolder: false,
          children: [],
          nodeId: node.id,
          content_type: node.content_type,
        })
      }
    } else {
      root.children.push({
        name: 'Unsorted',
        path: '/Unsorted',
        isFolder: true,
        children: itemsWithoutPath.map((node) => ({
          name: node.title,
          path: node.title,
          isFolder: false,
          children: [],
          nodeId: node.id,
          content_type: node.content_type,
        })),
      })
    }
  }

  return sortTree(root.children)
}

export function sortTree(nodes: TreeNode[]): TreeNode[] {
  const folders = nodes.filter((n) => n.isFolder).sort((a, b) => a.name.localeCompare(b.name))
  const files = nodes.filter((n) => !n.isFolder).sort((a, b) => a.name.localeCompare(b.name))
  return [
    ...folders.map((f) => ({ ...f, children: sortTree(f.children) })),
    ...files,
  ]
}

function TreeNodeRow({
  node,
  depth,
  selectedNodeId,
  onSelectNode,
}: {
  node: TreeNode
  depth: number
  selectedNodeId: string | null
  onSelectNode: (nodeId: string) => void
}) {
  const [expanded, setExpanded] = useState(depth < 1)

  if (node.isFolder) {
    return (
      <div>
        <button
          className="w-full text-left px-1 py-0.5 text-xs text-muted-foreground hover:bg-muted rounded flex items-center gap-0.5 cursor-pointer"
          style={{ paddingLeft: `${depth * 12 + 4}px` }}
          onClick={() => setExpanded(!expanded)}
        >
          <span className="text-[10px] w-3 shrink-0">{expanded ? '\u25BC' : '\u25B6'}</span>
          <span>{node.name}</span>
        </button>
        {expanded && (
          <div>
            {node.children.map((child) => (
              <TreeNodeRow
                key={child.path}
                node={child}
                depth={depth + 1}
                selectedNodeId={selectedNodeId}
                onSelectNode={onSelectNode}
              />
            ))}
          </div>
        )}
      </div>
    )
  }

  return (
    <button
      className={`w-full text-left px-1 py-0.5 text-sm rounded flex items-center gap-1 cursor-pointer ${
        selectedNodeId === node.nodeId ? 'bg-accent text-accent-foreground' : 'hover:bg-muted'
      }`}
      style={{ paddingLeft: `${depth * 12 + 4}px` }}
      onClick={() => node.nodeId && onSelectNode(node.nodeId)}
    >
      <span className="truncate flex-1">{node.name}</span>
      {node.content_type && (
        <span className="text-[10px] text-muted-foreground shrink-0">{node.content_type}</span>
      )}
    </button>
  )
}

export function FileTree() {
  const { nodes, selectedNodeId, selectNode, filterText, filterTypes } = useStore()

  const filteredNodes = useMemo(() => {
    let result = nodes
    if (filterText) {
      result = result.filter((n) => n.title.toLowerCase().includes(filterText.toLowerCase()))
    }
    if (filterTypes.length > 0) {
      result = result.filter((n) => filterTypes.includes(n.content_type))
    }
    return result
  }, [nodes, filterText, filterTypes])

  const tree = useMemo(() => buildTree(filteredNodes), [filteredNodes])

  return (
    <div className="flex-1 overflow-y-auto p-1">
      {tree.map((node) => (
        <TreeNodeRow
          key={node.path}
          node={node}
          depth={0}
          selectedNodeId={selectedNodeId}
          onSelectNode={selectNode}
        />
      ))}
      {tree.length === 0 && (
        <p className="text-xs text-muted-foreground px-2 py-1">No notes</p>
      )}
    </div>
  )
}
