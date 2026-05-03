import { Node } from '@tiptap/react'

declare module '@tiptap/core' {
  interface Commands<ReturnType> {
    noteEmbed: {
      insertNoteEmbed: (title: string, snippet: string, nodeId: string) => ReturnType
    }
  }
}

export const NoteEmbed = Node.create({
  name: 'noteEmbed',

  group: 'block',

  atom: true,

  addAttributes() {
    return {
      title: { default: '' },
      snippet: { default: '' },
      nodeId: { default: '' },
    }
  },

  parseHTML() {
    return [{ tag: 'div[data-type="note-embed"]' }]
  },

  renderHTML({ node }) {
    return [
      'div',
      {
        'data-type': 'note-embed',
        class: 'note-embed',
        'data-node-id': node.attrs.nodeId,
        'data-title': node.attrs.title,
      },
      ['div', { class: 'note-embed-header' }, `embedded: ${node.attrs.title}`],
      ['div', { class: 'note-embed-content' }, node.attrs.snippet],
    ]
  },

  addCommands() {
    return {
      insertNoteEmbed:
        (title: string, snippet: string, nodeId: string) =>
        ({ commands }) => {
          return commands.insertContent({
            type: this.name,
            attrs: { title, snippet, nodeId },
          })
        },
    }
  },
})
