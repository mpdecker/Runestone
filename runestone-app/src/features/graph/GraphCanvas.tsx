import { useEffect, useRef, useCallback, useMemo } from 'react'
import cytoscape, { type Core, type EventObject } from 'cytoscape'
import { useShallow } from 'zustand/react/shallow'
import { useStore } from '@/store'

const NODE_COLORS: Record<string, string> = {
  note: '#3b82f6',
  concept: '#10b981',
  entity: '#f59e0b',
  document: '#ef4444',
}

const EDGE_COLORS: Record<string, string> = {
  REFERENCES: '#94a3b8',
  CONTAINS: '#6366f1',
  RELATES_TO: '#8b5cf6',
  EXTRACTED_FROM: '#ec4899',
  LINKS_TO: '#14b8a6',
  TAGGED: '#f97316',
}

const MAX_RENDER_NODES = 500

export function GraphCanvas() {
  const containerRef = useRef<HTMLDivElement>(null)
  const cyRef = useRef<Core | null>(null)
  const {
    graphData,
    selectNode,
    selectedNodeId,
    loadLocalGraph,
    setGraphViewMode,
    graphViewMode,
    filterText,
    filterTypes,
    graphDepth,
    setGraphDepth,
  } = useStore(
    useShallow((s) => ({
      graphData: s.graphData,
      selectNode: s.selectNode,
      selectedNodeId: s.selectedNodeId,
      loadLocalGraph: s.loadLocalGraph,
      setGraphViewMode: s.setGraphViewMode,
      graphViewMode: s.graphViewMode,
      filterText: s.filterText,
      filterTypes: s.filterTypes,
      graphDepth: s.graphDepth,
      setGraphDepth: s.setGraphDepth,
    })),
  )

  const nodeCount = graphData?.nodes.length ?? 0
  const edgeCount = graphData?.edges.length ?? 0
  const graphSignature = useMemo(
    () => `${nodeCount}:${edgeCount}`,
    [nodeCount, edgeCount],
  )

  const updateGraph = useCallback(() => {
    if (!cyRef.current || !graphData) return

    const cy = cyRef.current
    const existingNodes = new Set(cy.nodes().map((n) => n.id()))
    const existingEdges = new Set(cy.edges().map((e) => e.id()))

    const graphNodeIds = new Set<string>()

    for (const node of graphData.nodes) {
      if (graphNodeIds.size >= MAX_RENDER_NODES) break
      const filtered = filterTypes.length > 0 && !filterTypes.includes(node.content_type)
      const searched = filterText && !node.title.toLowerCase().includes(filterText.toLowerCase())
      if (filtered || searched) continue

      graphNodeIds.add(node.id)

      if (!existingNodes.has(node.id)) {
        cy.add({
          group: 'nodes',
          data: {
            id: node.id,
            label: node.title,
            content_type: node.content_type,
          },
        })
      } else {
        const existing = cy.getElementById(node.id)
        existing.data('label', node.title)
        existing.data('content_type', node.content_type)
      }
    }

    for (const edge of graphData.edges) {
      if (!graphNodeIds.has(edge.source) || !graphNodeIds.has(edge.target)) continue

      const edgeId = `${edge.source}->${edge.target}:${edge.label}`
      if (!existingEdges.has(edgeId)) {
        cy.add({
          group: 'edges',
          data: {
            id: edgeId,
            source: edge.source,
            target: edge.target,
            label: edge.label,
          },
        })
      }
    }

    for (const n of cy.nodes()) {
      if (!graphNodeIds.has(n.id())) {
        cy.remove(n)
      }
    }

    for (const e of cy.edges()) {
      const src = e.data('source')
      const tgt = e.data('target')
      if (!graphNodeIds.has(src) || !graphNodeIds.has(tgt)) {
        cy.remove(e)
      }
    }

    cy.layout({
      name: 'cose',
      animate: true,
      animationDuration: 500,
      fit: true,
      padding: 30,
      nodeRepulsion: () => 8000,
      idealEdgeLength: () => 120,
    }).run()
  }, [graphData, filterText, filterTypes])

  useEffect(() => {
    if (!containerRef.current || cyRef.current) return

    const cy = cytoscape({
      container: containerRef.current,
      style: [
        {
          selector: 'node',
          style: {
            'background-color': (el) => {
              const t = el.data('content_type') as string
              return NODE_COLORS[t] || '#94a3b8'
            },
            'label': 'data(label)',
            'font-size': '10px',
            'text-valign': 'bottom',
            'text-halign': 'center',
            'color': '#cbd5e1',
            'text-outline-color': '#1e293b',
            'text-outline-width': 1,
            'width': 12,
            'height': 12,
          },
        },
        {
          selector: 'node:selected',
          style: {
            'border-width': 2,
            'border-color': '#fff',
            'width': 16,
            'height': 16,
          },
        },
        {
          selector: 'edge',
          style: {
            'width': 1,
            'line-color': (el) => {
              const t = el.data('label') as string
              return EDGE_COLORS[t] || '#64748b'
            },
            'target-arrow-color': (el) => {
              const t = el.data('label') as string
              return EDGE_COLORS[t] || '#64748b'
            },
            'target-arrow-shape': 'triangle',
            'curve-style': 'bezier',
            'opacity': 0.6,
          },
        },
      ],
      layout: { name: 'preset' },
      wheelSensitivity: 0.3,
    })

    cy.on('tap', 'node', (evt: EventObject) => {
      const nodeId = evt.target.id()
      selectNode(nodeId)
    })

    cy.on('dbltap', 'node', (evt: EventObject) => {
      const nodeId = evt.target.id()
      loadLocalGraph(nodeId)
    })

    cyRef.current = cy
  }, [selectNode, loadLocalGraph])

  useEffect(() => {
    updateGraph()
  }, [updateGraph, graphSignature])

  useEffect(() => {
    if (cyRef.current && selectedNodeId) {
      const node = cyRef.current.getElementById(selectedNodeId)
      if (node.length > 0) {
        cyRef.current.animate({
          center: { eles: node },
          zoom: 1.2,
          duration: 300,
        })
        node.select()
      }
    }
  }, [selectedNodeId])

  if (!graphData || graphData.nodes.length === 0) {
    return (
      <div className="flex-1 flex items-center justify-center text-muted-foreground text-sm">
        No graph data. Create nodes or scan a vault to populate the graph.
      </div>
    )
  }

  return (
    <div className="flex-1 flex flex-col min-h-0 bg-slate-950">
      <div className="border-b border-slate-800 px-3 py-1.5 flex items-center gap-3 text-xs flex-wrap">
        <span className="text-slate-400">
          {graphData.nodes.length > MAX_RENDER_NODES ? `${MAX_RENDER_NODES}+` : graphData.nodes.length} nodes, {graphData.edges.length} edges
        </span>
        <div className="flex gap-1">
          <button
            className={`px-2 py-0.5 rounded ${graphViewMode === 'global' ? 'bg-slate-700 text-slate-200' : 'text-slate-500 hover:text-slate-300'}`}
            onClick={() => setGraphViewMode('global')}
          >
            Global
          </button>
          <button
            className={`px-2 py-0.5 rounded ${graphViewMode === 'local' ? 'bg-slate-700 text-slate-200' : 'text-slate-500 hover:text-slate-300'}`}
            onClick={() => {
              const { selectedNodeId } = useStore.getState()
              if (selectedNodeId) loadLocalGraph(selectedNodeId)
            }}
          >
            Local
          </button>
        </div>
        {graphViewMode === 'local' && (
          <div className="flex items-center gap-1">
            <span className="text-slate-500">Depth:</span>
            {[1, 2, 3, 4, 5].map((d) => (
              <button
                key={d}
                className={`px-1.5 py-0.5 rounded text-[10px] ${graphDepth === d ? 'bg-slate-700 text-slate-200' : 'text-slate-500 hover:text-slate-300'}`}
                onClick={() => {
                  setGraphDepth(d)
                  const { selectedNodeId } = useStore.getState()
                  if (selectedNodeId) loadLocalGraph(selectedNodeId, d)
                }}
              >
                {d}
              </button>
            ))}
          </div>
        )}
        <span className="text-slate-500 text-[10px]">
          Double-click node for local view
        </span>
      </div>
      <div ref={containerRef} className="flex-1" />
    </div>
  )
}
