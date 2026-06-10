import { ReactRenderer } from '@tiptap/react'
import type { SuggestionOptions, SuggestionProps } from '@tiptap/suggestion'

interface NodeItem {
  id: string
  title: string
  content_type: string
}

function fuzzyMatch(pattern: string, text: string): number | null {
  const p = pattern.toLowerCase()
  const t = text.toLowerCase()
  let pi = 0
  let score = 0
  let prevMatch = -1
  for (let ti = 0; ti < t.length && pi < p.length; ti++) {
    if (t[ti] === p[pi]) {
      score += prevMatch === ti - 1 ? 2 : 1
      prevMatch = ti
      pi++
    }
  }
  return pi === p.length ? score : null
}

function WikiLinkList({
  items,
  selectedIndex,
  command,
}: {
  items: NodeItem[]
  selectedIndex: number
  command: (item: NodeItem) => void
}) {
  return (
    <div className="bg-card border rounded-lg shadow-lg p-1 max-h-48 overflow-y-auto min-w-[200px]" role="listbox" aria-label="Wiki link suggestions">
      {items.length > 0 ? (
        items.map((item, index) => (
          <button
            key={item.id}
            role="option"
            aria-selected={index === selectedIndex}
            className={`w-full text-left px-3 py-1.5 text-sm rounded flex items-center justify-between ${
              index === selectedIndex ? 'bg-accent text-accent-foreground' : 'hover:bg-accent'
            }`}
            onMouseDown={(e) => {
              e.preventDefault()
              command(item)
            }}
          >
            <span className="truncate">{item.title}</span>
            <span className="text-[10px] text-muted-foreground shrink-0 ml-2">{item.content_type}</span>
          </button>
        ))
      ) : (
        <p className="px-3 py-2 text-xs text-muted-foreground">No matching notes</p>
      )}
    </div>
  )
}

export function createWikiLinkSuggestion(
  getNodes: () => NodeItem[],
  onSelectNode: (nodeId: string) => void,
): Omit<SuggestionOptions<NodeItem>, 'editor'> {
  let component: ReactRenderer | null = null
  let el: HTMLElement | null = null
  let selectedIndex = 0
  let currentItems: NodeItem[] = []
  let latestProps: SuggestionProps<NodeItem> | null = null

  const renderList = () => {
    if (!component || !latestProps) return
    component.updateProps({
      items: currentItems,
      selectedIndex,
      command: latestProps.command,
    })
  }

  return {
    char: '[[',

    items: ({ query }) => {
      if (!query) return []
      const scored = getNodes()
        .map((n) => ({ node: n, score: fuzzyMatch(query, n.title) }))
        .filter((x): x is { node: NodeItem; score: number } => x.score !== null)
        .sort((a, b) => b.score - a.score)
        .slice(0, 8)
      return scored.map((s) => s.node)
    },

    render: () => {
      return {
        onStart: (props: SuggestionProps<NodeItem>) => {
          latestProps = props
          selectedIndex = 0
          currentItems = props.items
          component = new ReactRenderer(WikiLinkList, {
            props: { items: currentItems, selectedIndex, command: props.command },
            editor: props.editor,
          })

          el = document.createElement('div')
          el.className = 'wiki-link-suggestion-container'
          el.style.position = 'absolute'
          el.style.zIndex = '50'
          document.body.appendChild(el)

          if (component.element) {
            while (el.firstChild) el.removeChild(el.firstChild)
            el.appendChild(component.element)
          }

          if (props.clientRect) {
            const rect = props.clientRect()
            if (rect) {
              el.style.left = `${rect.left}px`
              el.style.top = `${rect.bottom + 4}px`
            }
          }
        },

        onUpdate(props: SuggestionProps<NodeItem>) {
          latestProps = props
          selectedIndex = 0
          currentItems = props.items
          renderList()
          if (el && props.clientRect) {
            const rect = props.clientRect()
            if (rect) {
              el.style.left = `${rect.left}px`
              el.style.top = `${rect.bottom + 4}px`
            }
          }
        },

        onKeyDown(props: { event: KeyboardEvent }) {
          if (props.event.key === 'Escape') {
            el?.remove()
            component?.destroy()
            component = null
            el = null
            return true
          }
          if (props.event.key === 'ArrowDown') {
            props.event.preventDefault()
            if (currentItems.length === 0) return true
            selectedIndex = Math.min(selectedIndex + 1, currentItems.length - 1)
            renderList()
            return true
          }
          if (props.event.key === 'ArrowUp') {
            props.event.preventDefault()
            if (currentItems.length === 0) return true
            selectedIndex = Math.max(selectedIndex - 1, 0)
            renderList()
            return true
          }
          if (props.event.key === 'Enter') {
            props.event.preventDefault()
            const item = currentItems[selectedIndex]
            if (item && latestProps) {
              latestProps.command(item)
              return true
            }
            return true
          }
          return false
        },

        onExit() {
          el?.remove()
          el = null
          component?.destroy()
          component = null
          latestProps = null
        },
      }
    },

    command: ({ editor, range, props }) => {
      editor
        .chain()
        .focus()
        .deleteRange(range)
        .insertContent([
          { type: 'wikiLink', attrs: { title: props.title, nodeId: props.id } },
          { type: 'text', text: ' ' },
        ])
        .run()

      onSelectNode(props.id)
    },
  }
}
