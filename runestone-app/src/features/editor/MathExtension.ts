import type { RawCommands } from '@tiptap/core'
import { Node } from '@tiptap/react'
import katex from 'katex'

declare module '@tiptap/core' {
  interface Commands<ReturnType> {
    mathInline: {
      insertMathInline: (formula: string) => ReturnType
    }
  }
}

export const MathInline = Node.create({
  name: 'mathInline',

  group: 'inline',

  inline: true,

  atom: true,

  addAttributes() {
    return {
      formula: {
        default: '',
      },
    }
  },

  parseHTML() {
    return [{ tag: 'span[data-type="math-inline"]' }]
  },

  renderHTML({ node }) {
    try {
      const html = katex.renderToString(node.attrs.formula, {
        throwOnError: false,
        displayMode: false,
      })
      return ['span', { 'data-type': 'math-inline' }, html]
    } catch {
      return ['span', { 'data-type': 'math-inline', class: 'math-error' }, node.attrs.formula]
    }
  },

  addCommands() {
    return {
      insertMathInline:
        (formula: string) =>
        ({ commands }: { commands: { insertContent: (content: unknown) => boolean } }) => {
          return commands.insertContent({
            type: this.name,
            attrs: { formula },
          })
        },
    } as Partial<RawCommands>
  },
})
