import { useCallback, useEffect, useRef, useState } from 'react'
import { useEditor, EditorContent } from '@tiptap/react'
import StarterKit from '@tiptap/starter-kit'
import { Suggestion } from '@tiptap/suggestion'
import CodeBlockLowlight from '@tiptap/extension-code-block-lowlight'
import TaskList from '@tiptap/extension-task-list'
import TaskItem from '@tiptap/extension-task-item'
import Table from '@tiptap/extension-table'
import TableRow from '@tiptap/extension-table-row'
import TableCell from '@tiptap/extension-table-cell'
import TableHeader from '@tiptap/extension-table-header'
import { common, createLowlight } from 'lowlight'
import { useStore } from '@/store'
import { Button } from '@/components/ui/button'
import { WikiLink } from './WikiLinkExtension'
import { createWikiLinkSuggestion } from './WikiLinkSuggestion'
import { MathInline } from './MathExtension'
import { MermaidDiagram } from './MermaidExtension'
import { NoteEmbed } from './NoteEmbedExtension'
import { createSlashCommands } from './SlashCommands'
import { Footnote } from './FootnoteExtension'
import { AudioRecorder } from './AudioRecorder'
import type { NodeListItem } from '@/lib/types'

const lowlight = createLowlight(common)

interface PreviewState {
  visible: boolean
  x: number
  y: number
  node: NodeListItem | null
  snippet: string | null
}

interface Props {
  secondary?: boolean
}

export function NoteEditor({ secondary = false }: Props) {
  const store = useStore()
  const currentNode = secondary ? store.secondaryNode : store.currentNode
  const selectedNodeId = secondary ? store.secondaryTabId : store.selectedNodeId
  const {
    updateNodeContent, saveNode, isEditorDirty, nodes, selectNode,
    readingMode, toggleReadingMode,
  } = store
  const saveTimerRef = useRef<number | null>(null)
  const [preview, setPreview] = useState<PreviewState>({ visible: false, x: 0, y: 0, node: null, snippet: null })
  const previewTimerRef = useRef<number | null>(null)

  const suggestionRef = useRef<ReturnType<typeof createWikiLinkSuggestion> | null>(null)

  if (!suggestionRef.current) {
    suggestionRef.current = createWikiLinkSuggestion(
      () => nodes.map((n) => ({ id: n.id, title: n.title, content_type: n.content_type })),
      (nodeId) => selectNode(nodeId),
    )
  }

  const editor = useEditor({
    extensions: [
      StarterKit.configure({
        codeBlock: false,
      }),
      CodeBlockLowlight.configure({
        lowlight,
      }),
      TaskList,
      TaskItem.configure({
        nested: true,
      }),
      MathInline,
      MermaidDiagram,
      NoteEmbed,
      Table.configure({ resizable: true }),
      TableRow,
      TableCell,
      TableHeader,
      Footnote,
      Suggestion.configure({
        ...suggestionRef.current,
      }),
      Suggestion.configure(
        createSlashCommands(() => editor),
      ),
      WikiLink.configure({
        HTMLAttributes: {
          class: 'wiki-link',
        },
      }),
    ],
    content: currentNode?.content || '',
    onUpdate: ({ editor }) => {
      const html = editor.getHTML()
      updateNodeContent(html)

      if (saveTimerRef.current) {
        clearTimeout(saveTimerRef.current)
      }
      saveTimerRef.current = window.setTimeout(() => {
        saveNode()
      }, 2000)
    },
  })

  useEffect(() => {
    if (editor && currentNode) {
      const currentContent = editor.getHTML()
      if (currentContent !== currentNode.content) {
        editor.commands.setContent(currentNode.content || '')
      }
    }
  }, [currentNode?.id])

  useEffect(() => {
    return () => {
      if (saveTimerRef.current) {
        clearTimeout(saveTimerRef.current)
      }
    }
  }, [])

  useEffect(() => {
    if (!editor) return
    const timer = setTimeout(async () => {
      try {
        const mermaid = (await import('mermaid')).default
        mermaid.initialize({ startOnLoad: false, theme: 'default' })
        const container = document.querySelector('.ProseMirror')
        if (container) {
          const elements = container.querySelectorAll('.mermaid:not([data-processed])')
          for (const el of elements) {
            try {
              const id = el.id
              const code = el.textContent || ''
              const { svg } = await mermaid.render(id + '-svg', code)
              el.innerHTML = svg
              el.setAttribute('data-processed', 'true')
            } catch {
              // skip invalid mermaid blocks
            }
          }
        }
      } catch {
        // mermaid not available
      }
    }, 300)
    return () => clearTimeout(timer)
  }, [editor?.getHTML()])

  useEffect(() => {
    if (editor) {
      editor.setEditable(!readingMode)
    }
  }, [readingMode, editor])

  const handleSave = useCallback(() => {
    if (saveTimerRef.current) {
      clearTimeout(saveTimerRef.current)
    }
    saveNode()
  }, [saveNode])

  const handleEditorClick = useCallback((e: React.MouseEvent) => {
    const target = e.target as HTMLElement
    const linkEl = target.closest('[data-type="wiki-link"]')
    if (linkEl) {
      const title = linkEl.getAttribute('data-title')
      if (title) {
        const matched = nodes.find((n) => n.title === title)
        if (matched) {
          selectNode(matched.id)
        }
      }
    }
  }, [nodes, selectNode])

  const handleMouseOver = useCallback((e: React.MouseEvent) => {
    const target = e.target as HTMLElement
    const linkEl = target.closest('[data-type="wiki-link"]')
    if (linkEl) {
      const title = linkEl.getAttribute('data-title')
      if (title) {
        const matched = nodes.find((n) => n.title === title)
        if (matched) {
          if (previewTimerRef.current) clearTimeout(previewTimerRef.current)
          previewTimerRef.current = window.setTimeout(async () => {
            const rect = linkEl.getBoundingClientRect()
            let snippet: string | null = null
            try {
              const { getNode } = await import('@/lib/api')
              const fullNode = await getNode(matched.id)
              snippet = fullNode.content
                .replace(/<[^>]+>/g, '')
                .replace(/\s+/g, ' ')
                .trim()
                .slice(0, 250)
            } catch {
              // snippet load failed
            }
            setPreview({
              visible: true,
              x: rect.left,
              y: rect.bottom + 4,
              node: matched,
              snippet,
            })
          }, 400)
        }
      }
    } else {
      if (previewTimerRef.current) clearTimeout(previewTimerRef.current)
      setPreview((p) => (p.visible ? { ...p, visible: false } : p))
    }
  }, [nodes])

  const handleMouseLeave = useCallback(() => {
    if (previewTimerRef.current) clearTimeout(previewTimerRef.current)
    setPreview((p) => (p.visible ? { ...p, visible: false } : p))
  }, [])

  if (!selectedNodeId || !currentNode) {
    return null
  }

  return (
    <div className="h-full flex flex-col min-h-0">
      <div className="border-b px-3 py-2 flex items-center justify-between bg-card shrink-0">
        <div className="flex items-center gap-2 min-w-0">
          <h1 className="font-semibold text-sm truncate">{currentNode.title}</h1>
          {isEditorDirty && (
            <span className="text-xs text-muted-foreground shrink-0">(unsaved)</span>
          )}
        </div>
        <div className="flex items-center gap-2 text-xs text-muted-foreground shrink-0">
          <AudioRecorder editor={editor} />
          <span>{currentNode.word_count} words</span>
          <Button variant="ghost" size="sm" onClick={toggleReadingMode} className="h-6 text-xs px-1">
            {readingMode ? 'Edit' : 'Read'}
          </Button>
          <Button variant="outline" size="sm" onClick={handleSave} className="h-6 text-xs">
            Save
          </Button>
        </div>
      </div>
      <div
        className="flex-1 overflow-y-auto relative"
        onClick={handleEditorClick}
        onMouseOver={handleMouseOver}
        onMouseLeave={handleMouseLeave}
      >
        <EditorContent
          editor={editor}
          className="prose prose-sm dark:prose-invert max-w-none p-4 min-h-full focus:outline-none"
        />
        {preview.visible && preview.node && (
          <div
            className="fixed z-50 w-72 p-3 bg-card border rounded-lg shadow-lg text-xs space-y-1"
            style={{ left: `${preview.x}px`, top: `${preview.y}px` }}
            onMouseEnter={() => {
              if (previewTimerRef.current) clearTimeout(previewTimerRef.current)
            }}
            onMouseLeave={() => setPreview((p) => ({ ...p, visible: false }))}
          >
            <div className="font-semibold truncate">{preview.node.title}</div>
            <div className="text-muted-foreground">{preview.node.content_type}</div>
            {preview.snippet && (
              <div className="text-muted-foreground leading-relaxed max-h-20 overflow-hidden">
                {preview.snippet}
              </div>
            )}
            <div className="text-muted-foreground text-[10px]">
              Updated: {preview.node.updated_at ? new Date(preview.node.updated_at).toLocaleDateString() : 'N/A'}
            </div>
          </div>
        )}
      </div>
    </div>
  )
}
