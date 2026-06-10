import type { RawCommands } from '@tiptap/core'
import { Node } from '@tiptap/react'

declare module '@tiptap/core' {
  interface Commands<ReturnType> {
    mermaidDiagram: {
      insertMermaidDiagram: (code: string) => ReturnType
    }
  }
}

export const MermaidDiagram = Node.create({
  name: 'mermaidDiagram',

  group: 'block',

  atom: true,

  addAttributes() {
    return {
      code: {
        default: '',
      },
    }
  },

  parseHTML() {
    return [{ tag: 'div[data-type="mermaid"]' }]
  },

  renderHTML({ node }) {
    const containerId = `mermaid-${Math.random().toString(36).slice(2, 8)}`
    return [
      'div',
      {
        'data-type': 'mermaid',
        'data-mermaid-id': containerId,
        class: 'mermaid-container',
      },
      ['div', { id: containerId, class: 'mermaid' }, node.attrs.code],
    ]
  },

  addCommands() {
    return {
      insertMermaidDiagram:
        (code: string) =>
        ({ commands }: { commands: { insertContent: (content: unknown) => boolean } }) => {
          return commands.insertContent({
            type: this.name,
            attrs: { code },
          })
        },
    } as Partial<RawCommands>
  },
})
