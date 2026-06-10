import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { useEditor, EditorContent, type Editor } from '@tiptap/react'
import StarterKit from '@tiptap/starter-kit'
import CodeBlockLowlight from '@tiptap/extension-code-block-lowlight'
import TaskList from '@tiptap/extension-task-list'
import TaskItem from '@tiptap/extension-task-item'
import { Table } from '@tiptap/extension-table'
import { TableRow } from '@tiptap/extension-table-row'
import { TableCell } from '@tiptap/extension-table-cell'
import { TableHeader } from '@tiptap/extension-table-header'
import { common, createLowlight } from 'lowlight'
import { useShallow } from 'zustand/react/shallow'
import { useStore } from '@/store'
import { Button } from '@/components/ui/button'
import { WikiLink } from './WikiLinkExtension'
import { createWikiLinkSuggestion } from './WikiLinkSuggestion'
import { WikiLinkSuggestionExtension } from './WikiLinkSuggestionExtension'
import { SlashCommandsExtension } from './SlashCommandsExtension'
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
  const {
    currentNode,
    secondaryNode,
    selectedNodeId,
    secondaryTabId,
    updateNodeContent,
    saveNode,
    isEditorDirty,
    isSecondaryEditorDirty,
    isSaving,
    isSecondarySaving,
    nodes,
    selectNode,
    readingMode,
    toggleReadingMode,
    fetchWikiLinkPreview,
  } = useStore(
    useShallow((s) => ({
      currentNode: secondary ? s.secondaryNode : s.currentNode,
      secondaryNode: s.secondaryNode,
      selectedNodeId: s.selectedNodeId,
      secondaryTabId: s.secondaryTabId,
      updateNodeContent: s.updateNodeContent,
      saveNode: s.saveNode,
      isEditorDirty: s.isEditorDirty,
      isSecondaryEditorDirty: s.isSecondaryEditorDirty,
      isSaving: s.isSaving,
      isSecondarySaving: s.isSecondarySaving,
      nodes: s.nodes,
      selectNode: s.selectNode,
      readingMode: s.readingMode,
      toggleReadingMode: s.toggleReadingMode,
      fetchWikiLinkPreview: s.fetchWikiLinkPreview,
    })),
  )
  const activeNode = secondary ? secondaryNode : currentNode
  const activeNodeId = secondary ? secondaryTabId : selectedNodeId
  const saveTimerRef = useRef<number | null>(null)
  const [preview, setPreview] = useState<PreviewState>({ visible: false, x: 0, y: 0, node: null, snippet: null })
  const previewTimerRef = useRef<number | null>(null)

  const editorRef = useRef<Editor | null>(null)

  const extensions = useMemo(
    () => [
      StarterKit.configure({ codeBlock: false }),
      CodeBlockLowlight.configure({ lowlight }),
      TaskList,
      TaskItem.configure({ nested: true }),
      MathInline,
      MermaidDiagram,
      NoteEmbed,
      Table.configure({ resizable: true }),
      TableRow,
      TableCell,
      TableHeader,
      Footnote,
      WikiLinkSuggestionExtension(
        createWikiLinkSuggestion(
          () => nodes.map((n) => ({ id: n.id, title: n.title, content_type: n.content_type })),
          (nodeId) => selectNode(nodeId),
        ),
      ),
      SlashCommandsExtension(createSlashCommands(() => editorRef.current)),
      WikiLink.configure({
        HTMLAttributes: { class: 'wiki-link' },
      }),
    ],
    [nodes, selectNode],
  )

  const editor = useEditor({
    extensions,
    content: activeNode?.content || '',
    onUpdate: ({ editor: ed }) => {
      const html = ed.getHTML()
      updateNodeContent(html, secondary)

      if (saveTimerRef.current) {
        clearTimeout(saveTimerRef.current)
      }
      saveTimerRef.current = window.setTimeout(() => {
        saveNode(secondary)
      }, 2000)
    },
  })

  useEffect(() => {
    editorRef.current = editor
  }, [editor])

  useEffect(() => {
    if (editor && activeNode) {
      const currentContent = editor.getHTML()
      if (currentContent !== activeNode.content) {
        editor.commands.setContent(activeNode.content || '')
      }
    }
  }, [activeNode?.id])

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
  }, [activeNodeId])

  useEffect(() => {
    if (editor) {
      editor.setEditable(!readingMode)
    }
  }, [readingMode, editor])

  const handleSave = useCallback(() => {
    if (saveTimerRef.current) {
      clearTimeout(saveTimerRef.current)
    }
    saveNode(secondary)
  }, [saveNode, secondary])

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
            const snippet = await fetchWikiLinkPreview(matched.id)
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
  }, [nodes, fetchWikiLinkPreview])

  const handleMouseLeave = useCallback(() => {
    if (previewTimerRef.current) clearTimeout(previewTimerRef.current)
    setPreview((p) => (p.visible ? { ...p, visible: false } : p))
  }, [])

  if (!activeNodeId || !activeNode) {
    return null
  }

  return (
    <div className="h-full flex flex-col min-h-0">
      <div className="border-b px-3 py-2 flex items-center justify-between bg-card shrink-0">
        <div className="flex items-center gap-2 min-w-0">
          <h1 className="font-semibold text-sm truncate">{activeNode.title}</h1>
          {(secondary ? isSecondaryEditorDirty : isEditorDirty) && (
            <span className="text-xs text-muted-foreground shrink-0">(unsaved)</span>
          )}
          {(secondary ? isSecondarySaving : isSaving) && (
            <span className="text-xs text-muted-foreground shrink-0">Saving...</span>
          )}
        </div>
        <div className="flex items-center gap-2 text-xs text-muted-foreground shrink-0">
          <AudioRecorder editor={editor} />
          <span>{activeNode.word_count} words</span>
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
