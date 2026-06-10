import type { RawCommands } from '@tiptap/core'
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
        ({ commands }: { commands: { insertContent: (content: unknown) => boolean } }) =>
          commands.insertContent({
            type: this.name,
            attrs: { title },
          }),
    } as Partial<RawCommands>
  },

  addKeyboardShortcuts() {
    return {}
  },
})
