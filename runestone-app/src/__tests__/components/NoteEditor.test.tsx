import { describe, it, expect, beforeEach, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import { NoteEditor } from '@/features/editor'
import { useStore } from '@/store'

vi.mock('@tiptap/react', () => ({
  useEditor: () => null,
  EditorContent: () => null,
  Node: {
    create: (config: Record<string, unknown>) => ({
      ...config,
      configure: () => ({ name: 'wikiLink', ...config }),
    }),
  },
}))

vi.mock('@tiptap/starter-kit', () => ({
  default: {
    configure: () => ({}),
  },
}))

vi.mock('@tiptap/suggestion', () => ({
  Suggestion: {
    configure: () => ({}),
  },
}))

vi.mock('@tiptap/extension-code-block-lowlight', () => ({
  default: {
    configure: () => ({}),
  },
}))

vi.mock('@tiptap/extension-task-list', () => ({
  default: {},
}))

vi.mock('@tiptap/extension-task-item', () => ({
  default: {
    configure: () => ({}),
  },
}))

vi.mock('lowlight', () => ({
  common: {},
  createLowlight: () => ({}),
}))

vi.mock('katex', () => ({
  default: {
    renderToString: () => '<span class="math">x</span>',
  },
}))

vi.mock('@/features/editor/MathExtension', () => ({
  MathInline: {},
}))

vi.mock('@/features/editor/MermaidExtension', () => ({
  MermaidDiagram: {},
}))

vi.mock('@/features/editor/NoteEmbedExtension', () => ({
  NoteEmbed: {},
}))

vi.mock('@/features/editor/SlashCommands', () => ({
  createSlashCommands: () => ({
    char: '/',
    items: () => [],
    render: () => ({
      onStart: () => {},
      onUpdate: () => {},
      onKeyDown: () => false,
      onExit: () => {},
    }),
    command: () => {},
  }),
}))

vi.mock('@/features/editor/FootnoteExtension', () => ({
  Footnote: {},
}))

vi.mock('@tiptap/extension-table', () => ({
  default: {
    configure: () => ({}),
  },
}))

vi.mock('@tiptap/extension-table-row', () => ({
  default: {},
}))

vi.mock('@tiptap/extension-table-cell', () => ({
  default: {},
}))

vi.mock('@tiptap/extension-table-header', () => ({
  default: {},
}))

vi.mock('@/features/editor/WikiLinkSuggestion', () => ({
  createWikiLinkSuggestion: () => ({
    char: '',
    items: () => [],
    render: () => ({
      onStart: () => {},
      onUpdate: () => {},
      onKeyDown: () => false,
      onExit: () => {},
    }),
    command: () => {},
  }),
}))

function resetStore() {
  useStore.setState({
    currentNode: null,
    selectedNodeId: null,
    isEditorDirty: false,
    vaults: [],
    selectedVaultId: null,
    nodes: [],
    isLoading: false,
    error: null,
    graphData: null,
    graphViewMode: 'global',
    backlinks: [],
    showBacklinks: false,
    filterText: '',
    filterTypes: [],
    searchQuery: '',
    searchResults: null,
    searchLoading: false,
    showSearch: false,
    similarNodes: [],
    pendingExtractions: [],
    showExtractions: false,
    extractionResults: [],
    darkMode: false,
    nodeSummary: null,
    summaryLoading: false,
    suggestedLinks: [],
    showCommandPalette: false,
    showChat: false,
    chatMessages: [],
    chatLoading: false,
    chatCitations: [],
    chatAnswer: null,
    tagSuggestions: [],
    importResult: null,
  })
}

const sampleNode = {
  id: 'n-1',
  vault_id: 'v-1',
  title: 'My Note',
  content: '<p>Hello world</p>',
  content_type: 'note',
  file_path: null,
  metadata: {},
  word_count: 42,
  created_at: null,
  updated_at: null,
}

describe('NoteEditor', () => {
  beforeEach(() => {
    resetStore()
  })

  it('renders nothing when no node selected', () => {
    const { container } = render(<NoteEditor />)
    expect(container.innerHTML).toBe('')
  })

  it('renders nothing when no currentNode', () => {
    useStore.setState({ selectedNodeId: 'n-1', currentNode: null })
    const { container } = render(<NoteEditor />)
    expect(container.innerHTML).toBe('')
  })

  it('renders node title when node selected', () => {
    useStore.setState({ selectedNodeId: 'n-1', currentNode: sampleNode })
    render(<NoteEditor />)
    expect(screen.getByText('My Note')).toBeInTheDocument()
  })

  it('shows word count', () => {
    useStore.setState({ selectedNodeId: 'n-1', currentNode: sampleNode })
    render(<NoteEditor />)
    expect(screen.getByText('42 words')).toBeInTheDocument()
  })

  it('shows Save button', () => {
    useStore.setState({ selectedNodeId: 'n-1', currentNode: sampleNode })
    render(<NoteEditor />)
    expect(screen.getByText('Save')).toBeInTheDocument()
  })

  it('shows unsaved indicator when dirty', () => {
    useStore.setState({
      selectedNodeId: 'n-1',
      currentNode: sampleNode,
      isEditorDirty: true,
    })
    render(<NoteEditor />)
    expect(screen.getByText('(unsaved)')).toBeInTheDocument()
  })

  it('does not show unsaved when clean', () => {
    useStore.setState({
      selectedNodeId: 'n-1',
      currentNode: sampleNode,
      isEditorDirty: false,
    })
    render(<NoteEditor />)
    expect(screen.queryByText('(unsaved)')).not.toBeInTheDocument()
  })
})
