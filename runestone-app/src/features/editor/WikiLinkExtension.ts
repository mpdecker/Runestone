import { Node } from '@tiptap/react'

export interface WikiLinkOptions {
  HTMLAttributes: Record<string, unknown>
  onLinkClick?: (nodeId: string, title: string) => void
}

declare module '@tiptap/core' {
  interface Commands<ReturnType> {
    wikiLink: {
      insertWikiLink: (title: string) => ReturnType
    }
  }
}

export const WikiLink = Node.create<WikiLinkOptions>({
  name: 'wikiLink',

  group: 'inline',

  inline: true,

  atom: true,

  addOptions() {
    return {
      HTMLAttributes: {},
      onLinkClick: undefined,
    }
  },

  addAttributes() {
    return {
      title: {
        default: null,
        parseHTML: (element) => element.getAttribute('data-title'),
        renderHTML: (attributes) => {
          if (!attributes.title) return {}
          return { 'data-title': attributes.title }
        },
      },
      nodeId: {
        default: null,
        parseHTML: (element) => element.getAttribute('data-node-id'),
        renderHTML: (attributes) => {
          if (!attributes.nodeId) return {}
          return { 'data-node-id': attributes.nodeId }
        },
      },
      blockRef: {
        default: null,
        parseHTML: (element) => element.getAttribute('data-block-ref'),
        renderHTML: (attributes) => {
          if (!attributes.blockRef) return {}
          return { 'data-block-ref': attributes.blockRef }
        },
      },
    }
  },

  parseHTML() {
    return [
      {
        tag: 'span[data-type="wiki-link"]',
      },
    ]
  },

  renderHTML({ HTMLAttributes, node }) {
    return [
      'span',
      {
        'data-type': 'wiki-link',
        class: 'wiki-link',
        title: `Navigate to: ${node.attrs.title}`,
        ...HTMLAttributes,
      },
      `[[${node.attrs.title}]]`,
    ]
  },

  addCommands() {
    return {
      insertWikiLink:
        (title: string) =>
        ({ editor, commands }) => {
          const nodes = editor.state.doc.descendants((node) => {
            if (node.type.name === this.name) {
              return node
            }
            return false
          })

          if (!nodes) return false

          let nodeId: string | null = null
          for (const entry of nodes) {
            if (entry.node.type.name === this.name && entry.node.attrs.title === title) {
              return true
            }
          }

          return commands.insertContent({
            type: this.name,
            attrs: { title, nodeId: nodeId || undefined },
          })
        },
    }
  },

  addKeyboardShortcuts() {
    return {}
  },
})
