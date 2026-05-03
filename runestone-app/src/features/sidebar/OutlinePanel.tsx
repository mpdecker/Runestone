import { useStore } from '@/store'

interface HeadingItem {
  level: number
  text: string
  pos: number
}

function extractHeadings(): HeadingItem[] {
  const editorEl = document.querySelector('.ProseMirror') as HTMLElement | null
  if (!editorEl) return []

  const headings: HeadingItem[] = []
  const elements = editorEl.querySelectorAll('h1, h2, h3, h4, h5, h6')

  elements.forEach((el) => {
    const level = parseInt(el.tagName[1])
    const text = el.textContent || ''
    const pos = (el as HTMLElement).offsetTop
    headings.push({ level, text, pos })
  })

  return headings
}

function scrollToHeading(pos: number) {
  const editorEl = document.querySelector('.ProseMirror')?.closest('.overflow-y-auto')
  if (editorEl) {
    editorEl.scrollTo({ top: pos - 20, behavior: 'smooth' })
  }
}

export function OutlinePanel() {
  const { selectedNodeId, currentNode } = useStore()

  if (!selectedNodeId || !currentNode) return null

  const headings = extractHeadings()

  if (headings.length === 0) {
    return (
      <div className="border-t p-2 space-y-1">
        <p className="text-[10px] text-muted-foreground uppercase tracking-wider">Outline</p>
        <p className="text-[10px] text-muted-foreground px-1">No headings in this note</p>
      </div>
    )
  }

  return (
    <div className="border-t p-2 space-y-0.5">
      <p className="text-[10px] text-muted-foreground uppercase tracking-wider">Outline</p>
      <div className="max-h-40 overflow-y-auto space-y-0.5">
        {extractHeadings().length === 0 ? (
          <p className="text-[10px] text-muted-foreground px-1">No headings</p>
        ) : (
          extractHeadings().map((h, i) => (
            <button
              key={i}
              className="w-full text-left px-1 py-0.5 text-xs text-muted-foreground hover:bg-muted rounded truncate block"
              style={{ paddingLeft: `${(h.level - 1) * 8 + 4}px` }}
              onClick={() => scrollToHeading(h.pos)}
            >
              {h.text}
            </button>
          ))
        )}
      </div>
    </div>
  )
}
