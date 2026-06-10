import { Extension } from '@tiptap/core'
import { Suggestion } from '@tiptap/suggestion'
import type { SuggestionOptions } from '@tiptap/suggestion'

export function SlashCommandsExtension(
  options: Omit<SuggestionOptions, 'editor'>,
) {
  return Extension.create({
    name: 'slashCommands',

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
