import { useEffect, useRef, useState, useMemo } from 'react'
import { useStore } from '@/store'

interface Command {
  id: string
  label: string
  shortcut: string
  action: () => void
}

function fuzzyMatch(pattern: string, text: string): number | null {
  const p = pattern.toLowerCase()
  const t = text.toLowerCase()
  let pi = 0
  let score = 0
  let prevMatch = -1

  for (let ti = 0; ti < t.length && pi < p.length; ti++) {
    if (t[ti] === p[pi]) {
      if (prevMatch === ti - 1) {
        score += 2
      } else {
        score += 1
      }
      prevMatch = ti
      pi++
    }
  }

  if (pi === p.length) return score
  return null
}

export function CommandPalette() {
  const { showCommandPalette, toggleCommandPalette, createNode, toggleSearch, toggleExtractions, nodes, selectNode, registeredCommands } = useStore()
  const inputRef = useRef<HTMLInputElement>(null)
  const [query, setQuery] = useState('')
  const [selectedIndex, setSelectedIndex] = useState(0)

  useEffect(() => {
    if (showCommandPalette && inputRef.current) {
      inputRef.current.focus()
      setQuery('')
      setSelectedIndex(0)
    }
  }, [showCommandPalette])

  const baseCommands: Command[] = useMemo(() => {
    const cmds: Command[] = [
      { id: 'new-note', label: 'New Note', shortcut: 'Ctrl+N', action: () => { toggleCommandPalette(); createNode('Untitled') } },
      { id: 'search', label: 'Search', shortcut: 'Ctrl+K', action: () => { toggleCommandPalette(); toggleSearch() } },
      { id: 'extractions', label: 'Extractions', shortcut: 'Ctrl+Shift+E', action: () => { toggleCommandPalette(); toggleExtractions() } },
    ]
    if (registeredCommands) {
      for (const cmd of registeredCommands) {
        cmds.push({
          id: cmd.id,
          label: cmd.label,
          shortcut: '',
          action: () => { toggleCommandPalette(); cmd.handler() },
        })
      }
    }
    return cmds
  }, [registeredCommands, toggleCommandPalette, createNode, toggleSearch, toggleExtractions])

  const filteredNodes = useMemo(() => {
    if (!query.trim()) return []
    const scored = nodes
      .map((n) => {
        const score = fuzzyMatch(query, n.title)
        return { node: n, score }
      })
      .filter((x): x is { node: typeof nodes[0]; score: number } => x.score !== null)
      .sort((a, b) => b.score - a.score)
      .slice(0, 10)
    return scored
  }, [query, nodes])

  const filteredCommands = useMemo(() => {
    if (!query.trim()) return baseCommands.map((cmd) => ({ cmd, score: 1 }))
    const scored = baseCommands
      .map((c) => {
        const score = fuzzyMatch(query, c.label)
        return { cmd: c, score }
      })
      .filter((x): x is { cmd: Command; score: number } => x.score !== null)
      .sort((a, b) => b.score - a.score)
    return scored
  }, [query])

  const totalItems = filteredCommands.length + filteredNodes.length

  useEffect(() => {
    setSelectedIndex(0)
  }, [query])

  if (!showCommandPalette) return null

  const getItemAtIndex = (idx: number) => {
    if (idx < filteredCommands.length) {
      return { type: 'command' as const, cmd: filteredCommands[idx].cmd }
    }
    const nodeIdx = idx - filteredCommands.length
    if (nodeIdx < filteredNodes.length) {
      return { type: 'node' as const, node: filteredNodes[nodeIdx].node }
    }
    return null
  }

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Escape') {
      toggleCommandPalette()
    } else if (e.key === 'ArrowDown') {
      e.preventDefault()
      setSelectedIndex((prev) => Math.min(prev + 1, totalItems - 1))
    } else if (e.key === 'ArrowUp') {
      e.preventDefault()
      setSelectedIndex((prev) => Math.max(prev - 1, 0))
    } else if (e.key === 'Enter') {
      e.preventDefault()
      const item = getItemAtIndex(selectedIndex)
      if (item?.type === 'command') {
        item.cmd.action()
      } else if (item?.type === 'node') {
        toggleCommandPalette()
        selectNode(item.node.id)
      }
    }
  }

  return (
    <div className="fixed inset-0 z-50 flex items-start justify-center pt-[15vh] bg-black/50" onClick={toggleCommandPalette}>
      <div className="w-96 bg-card border rounded-lg shadow-lg overflow-hidden" onClick={(e) => e.stopPropagation()}>
        <input
          ref={inputRef}
          className="w-full px-4 py-3 text-sm bg-transparent border-b outline-none"
          placeholder="Search notes or commands..."
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          onKeyDown={handleKeyDown}
        />
        <div className="max-h-64 overflow-y-auto p-1">
          {filteredCommands.length > 0 && (
            <>
              {!query.trim() && (
                <p className="px-3 py-1 text-[10px] text-muted-foreground uppercase tracking-wider">Commands</p>
              )}
              {filteredCommands.map(({ cmd }, i) => {
                const isSelected = i === selectedIndex
                return (
                  <button
                    key={cmd.id}
                    className={`w-full flex items-center justify-between px-3 py-2 text-sm rounded ${isSelected ? 'bg-accent text-accent-foreground' : 'hover:bg-muted'}`}
                    onClick={cmd.action}
                  >
                    <span>{cmd.label}</span>
                    <span className="text-xs text-muted-foreground">{cmd.shortcut}</span>
                  </button>
                )
              })}
            </>
          )}
          {filteredNodes.length > 0 && (
            <>
              <p className="px-3 py-1 text-[10px] text-muted-foreground uppercase tracking-wider">Notes</p>
              {filteredNodes.map(({ node }, i) => {
                const globalIdx = filteredCommands.length + i
                const isSelected = globalIdx === selectedIndex
                return (
                  <button
                    key={node.id}
                    className={`w-full flex items-center justify-between px-3 py-2 text-sm rounded ${isSelected ? 'bg-accent text-accent-foreground' : 'hover:bg-muted'}`}
                    onClick={() => { toggleCommandPalette(); selectNode(node.id) }}
                  >
                    <span className="truncate">{node.title}</span>
                    <span className="text-[10px] text-muted-foreground shrink-0 ml-2">{node.content_type}</span>
                  </button>
                )
              })}
            </>
          )}
          {query.trim() && filteredCommands.length === 0 && filteredNodes.length === 0 && (
            <p className="px-3 py-4 text-sm text-muted-foreground text-center">No results found</p>
          )}
        </div>
      </div>
    </div>
  )
}
