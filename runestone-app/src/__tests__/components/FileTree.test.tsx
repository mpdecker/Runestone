import { describe, it, expect } from 'vitest'
import { buildTree, sortTree } from '@/features/sidebar/FileTree'

describe('buildTree', () => {
  it('builds tree from file paths correctly', () => {
    const nodes = [
      { id: 'n-1', title: 'Note A', content_type: 'note', file_path: 'projects/proj-a/notes/a.md' },
      { id: 'n-2', title: 'Note B', content_type: 'note', file_path: 'projects/proj-b/notes/b.md' },
    ]

    const tree = buildTree(nodes)

    expect(tree).toHaveLength(1)
    expect(tree[0].name).toBe('projects')
    expect(tree[0].isFolder).toBe(true)
    expect(tree[0].children).toHaveLength(2)
  })

  it('puts nodes without file_path in Unsorted folder', () => {
    const nodes = [
      { id: 'n-1', title: 'Orphan', content_type: 'note', file_path: null },
    ]

    const tree = buildTree(nodes)

    const unsorted = tree.find((n) => n.name === 'Unsorted')
    expect(unsorted).toBeDefined()
    expect(unsorted!.children[0].name).toBe('Orphan')
  })

  it('handles empty input', () => {
    const tree = buildTree([])

    expect(tree).toEqual([])
  })

  it('preserves node metadata on file nodes', () => {
    const nodes = [
      { id: 'node-123', title: 'Title', content_type: 'entity', file_path: 'projects/myproject/index.md' },
    ]

    const tree = buildTree(nodes)

    expect(tree[0].name).toBe('projects')
    expect(tree[0].children[0].name).toBe('myproject')
    expect(tree[0].children[0].children[0].name).toBe('Title')
    expect(tree[0].children[0].children[0].nodeId).toBe('node-123')
    expect(tree[0].children[0].children[0].content_type).toBe('entity')
  })

  it('handles Windows-style backslash paths', () => {
    const nodes = [
      { id: 'n-1', title: 'Win Note', content_type: 'note', file_path: 'C:\\Users\\docs\\note.md' },
    ]

    const tree = buildTree(nodes)

    expect(tree[0].name).toBe('C:')
    expect(tree[0].children[0].name).toBe('Users')
    expect(tree[0].children[0].children[0].name).toBe('docs')
  })
})

describe('sortTree', () => {
  it('sorts folders before files', () => {
    const tree = [
      { name: 'b-file', path: 'b', isFolder: false, children: [], nodeId: '1' },
      { name: 'a-folder', path: 'a', isFolder: true, children: [], nodeId: '2' },
    ]

    const sorted = sortTree(tree)

    expect(sorted[0].name).toBe('a-folder')
    expect(sorted[1].name).toBe('b-file')
  })

  it('sorts folders alphabetically', () => {
    const tree = [
      { name: 'z-folder', path: 'z', isFolder: true, children: [], nodeId: '1' },
      { name: 'a-folder', path: 'a', isFolder: true, children: [], nodeId: '2' },
    ]

    const sorted = sortTree(tree)

    expect(sorted[0].name).toBe('a-folder')
    expect(sorted[1].name).toBe('z-folder')
  })

  it('sorts files alphabetically', () => {
    const tree = [
      { name: 'Zebra', path: 'z', isFolder: false, children: [], nodeId: '1' },
      { name: 'Apple', path: 'a', isFolder: false, children: [], nodeId: '2' },
    ]

    const sorted = sortTree(tree)

    expect(sorted[0].name).toBe('Apple')
    expect(sorted[1].name).toBe('Zebra')
  })
})
