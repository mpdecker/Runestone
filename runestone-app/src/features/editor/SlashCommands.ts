import type { SuggestionOptions } from '@tiptap/suggestion'

interface SlashCommand {
  id: string
  label: string
  description: string
  action: () => void
}

function fuzzyMatch(pattern: string, text: string): boolean {
  return text.toLowerCase().includes(pattern.toLowerCase())
}

function renderDropdown(items: SlashCommand[], container: HTMLElement) {
  container.innerHTML = ''
  items.forEach((cmd) => {
    const btn = document.createElement('button')
    btn.className = 'w-full text-left px-3 py-1.5 text-sm rounded hover:bg-accent flex items-center justify-between'
    btn.innerHTML = `<span>${cmd.label}</span><span class="text-[10px] text-muted-foreground ml-2">${cmd.description}</span>`
    btn.onmousedown = (e) => {
      e.preventDefault()
      cmd.action()
    }
    container.appendChild(btn)
  })
}

const BASE_COMMANDS: SlashCommand[] = [
  { id: 'h1', label: 'Heading 1', description: 'Large heading', action: () => {} },
  { id: 'h2', label: 'Heading 2', description: 'Medium heading', action: () => {} },
  { id: 'h3', label: 'Heading 3', description: 'Small heading', action: () => {} },
  { id: 'bullet', label: 'Bullet List', description: 'Unordered list', action: () => {} },
  { id: 'ordered', label: 'Numbered List', description: 'Ordered list', action: () => {} },
  { id: 'quote', label: 'Blockquote', description: 'Quote block', action: () => {} },
  { id: 'code', label: 'Code Block', description: 'Code with highlighting', action: () => {} },
  { id: 'divider', label: 'Divider', description: 'Horizontal rule', action: () => {} },
  { id: 'task', label: 'Task List', description: 'Checkable task items', action: () => {} },
]

function bindEditorCommands(editor: NonNullable<ReturnType<typeof import('@tiptap/react').useEditor>>) {
  BASE_COMMANDS.forEach((cmd) => {
    switch (cmd.id) {
      case 'h1': cmd.action = () => editor.chain().focus().toggleHeading({ level: 1 }).run(); break
      case 'h2': cmd.action = () => editor.chain().focus().toggleHeading({ level: 2 }).run(); break
      case 'h3': cmd.action = () => editor.chain().focus().toggleHeading({ level: 3 }).run(); break
      case 'bullet': cmd.action = () => editor.chain().focus().toggleBulletList().run(); break
      case 'ordered': cmd.action = () => editor.chain().focus().toggleOrderedList().run(); break
      case 'quote': cmd.action = () => editor.chain().focus().toggleBlockquote().run(); break
      case 'code': cmd.action = () => editor.chain().focus().toggleCodeBlock().run(); break
      case 'divider': cmd.action = () => editor.chain().focus().setHorizontalRule().run(); break
      case 'task': cmd.action = () => editor.chain().focus().toggleTaskList().run(); break
    }
  })
}

export function createSlashCommands(
  getEditor: () => NonNullable<ReturnType<typeof import('@tiptap/react').useEditor>> | null,
): Omit<SuggestionOptions<SlashCommand>, 'editor'> {
  return {
    char: '/',

    items: ({ query }) => {
      if (!query) return BASE_COMMANDS.slice(0, 6)
      return BASE_COMMANDS.filter((c) => fuzzyMatch(query, c.label))
    },

    render: () => {
      let container: HTMLElement
      let currentItems: SlashCommand[] = []

      return {
        onStart: (props) => {
          currentItems = props.items
          const editor = getEditor()
          if (editor) bindEditorCommands(editor)

          container = document.createElement('div')
          container.className = 'bg-card border rounded-lg shadow-lg p-1 max-h-56 overflow-y-auto min-w-[200px] fixed z-50'
          document.body.appendChild(container)

          if (props.clientRect) {
            const rect = props.clientRect()
            if (rect) {
              container.style.left = `${rect.left}px`
              container.style.top = `${rect.bottom + 4}px`
            }
          }
          renderDropdown(props.items, container)
        },

        onUpdate(props) {
          currentItems = props.items
          const editor = getEditor()
          if (editor) bindEditorCommands(editor)

          if (container && props.clientRect) {
            const rect = props.clientRect()
            if (rect) {
              container.style.left = `${rect.left}px`
              container.style.top = `${rect.bottom + 4}px`
            }
          }
          if (container) renderDropdown(props.items, container)
        },

        onKeyDown(props) {
          if (props.event.key === 'Escape') {
            container?.remove()
            return true
          }
          if (props.event.key === 'Enter' && currentItems.length > 0) {
            const editor = getEditor()
            if (editor) bindEditorCommands(editor)
            currentItems[0].action()
            container?.remove()
            return true
          }
          return false
        },

        onExit() {
          container?.remove()
        },
      }
    },

    command: ({ editor, range, props }) => {
      editor.chain().focus().deleteRange(range).run()
      bindEditorCommands(editor as NonNullable<ReturnType<typeof import('@tiptap/react').useEditor>>)
      props.action()
    },
  }
}
