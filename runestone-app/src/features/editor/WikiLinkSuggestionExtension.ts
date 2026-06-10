import { Extension } from '@tiptap/core'
import { Suggestion } from '@tiptap/suggestion'
import type { SuggestionOptions } from '@tiptap/suggestion'

export function WikiLinkSuggestionExtension(
  options: Omit<SuggestionOptions, 'editor'>,
) {
  return Extension.create({
    name: 'wikiLinkSuggestion',

    addProseMirrorPlugins() {
      return [
        Suggestion({
          editor: this.editor,
          ...options,
        }),
      ]
    },
  })
}
