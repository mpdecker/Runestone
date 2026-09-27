import { Extension } from '@tiptap/core'
import { Suggestion } from '@tiptap/suggestion'
import { PluginKey } from '@tiptap/pm/state'
import type { SuggestionOptions } from '@tiptap/suggestion'

export function SlashCommandsExtension(options: Omit<SuggestionOptions, 'editor'>) {
  return Extension.create({
    name: 'slashCommands',

    addProseMirrorPlugins() {
      return [
        Suggestion({
          pluginKey: new PluginKey('slashCommands'),
          editor: this.editor,
          ...options,
        }),
      ]
    },
  })
}
