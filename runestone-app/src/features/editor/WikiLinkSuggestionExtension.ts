import { Extension } from '@tiptap/core'
import { Suggestion } from '@tiptap/suggestion'
import { PluginKey } from '@tiptap/pm/state'
import type { SuggestionOptions } from '@tiptap/suggestion'

export function WikiLinkSuggestionExtension(options: Omit<SuggestionOptions, 'editor'>) {
  return Extension.create({
    name: 'wikiLinkSuggestion',

    addProseMirrorPlugins() {
      return [
        Suggestion({
          pluginKey: new PluginKey('wikiLinkSuggestion'),
          editor: this.editor,
          ...options,
        }),
      ]
    },
  })
}
