import { Node } from '@tiptap/react'

export const Footnote = Node.create({
  name: 'footnote',

  group: 'inline',

  inline: true,

  atom: true,

  addAttributes() {
    return {
      id: { default: '' },
      content: { default: '' },
    }
  },

  parseHTML() {
    return [{ tag: 'sup[data-type="footnote"]' }]
  },

  renderHTML({ node }) {
    return [
      'sup',
      {
        'data-type': 'footnote',
        'data-footnote-id': node.attrs.id,
        class: 'footnote-ref',
        title: node.attrs.content,
      },
      `[${node.attrs.id}]`,
    ]
  },
})
