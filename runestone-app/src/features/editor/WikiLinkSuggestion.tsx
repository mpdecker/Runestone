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

function WikiLinkList({ items, command }: { items: NodeItem[]; command: (item: NodeItem) => void }) {
  return (
    <div className="bg-card border rounded-lg shadow-lg p-1 max-h-48 overflow-y-auto min-w-[200px]">
      {items.length > 0 ? (
        items.map((item) => (
          <button
            key={item.id}
            className="w-full text-left px-3 py-1.5 text-sm rounded hover:bg-accent flex items-center justify-between"
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
          component = new ReactRenderer(WikiLinkList, {
            props,
            editor: props.editor,
          })

          el = document.createElement('div')
          el.className = 'wiki-link-suggestion-container'
          el.style.position = 'absolute'
          el.style.zIndex = '50'
          document.body.appendChild(el)

          const renderRoot = (el as unknown as { render: (node: React.ReactElement) => void })
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
          component?.updateProps(props as unknown as Record<string, unknown>)
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
          return false
        },

        onExit() {
          el?.remove()
          el = null
          component?.destroy()
          component = null
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
