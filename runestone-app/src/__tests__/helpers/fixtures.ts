import type { Node, NodeListItem } from '@/lib/types'
import type { PluginInfo } from '@/lib/plugin-types'

export function makeNodeListItem(overrides: Partial<NodeListItem> = {}): NodeListItem {
  return {
    id: 'node-1',
    title: 'Test Note',
    content_type: 'note',
    file_path: null,
    updated_at: null,
    ...overrides,
  }
}

export function makeNode(overrides: Partial<Node> = {}): Node {
  return {
    id: 'node-1',
    vault_id: 'vault-1',
    title: 'Test Note',
    content: '<p>Hello</p>',
    content_type: 'note',
    file_path: null,
    metadata: null,
    word_count: 1,
    created_at: null,
    updated_at: null,
    ...overrides,
  }
}

export function makePluginInfo(overrides: Partial<PluginInfo> = {}): PluginInfo {
  return {
    name: 'test-plugin',
    version: '1.0.0',
    path: '/plugins/test',
    main_file: 'main.js',
    description: 'Test plugin',
    author: 'Runestone',
    ...overrides,
  }
}
