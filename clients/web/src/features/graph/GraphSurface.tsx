import { useState } from "react"
import {
  Background,
  BackgroundVariant,
  Controls,
  MiniMap,
  ReactFlow,
  useReactFlow,
  type Connection,
  type Edge,
  type Node,
} from "@xyflow/react"

import { GraphContextMenu, type GraphMenuState } from "./GraphContextMenu"
import { parseNodeId, type DraftKind, type NodeKind } from "./model"
import { edgeTypes } from "./edgeTypes"
import { nodeTypes } from "./nodeTypes"
import { useGraphStore } from "./state/graphStore"

import "./graph.css"

const MINIMAP_COLORS: Record<string, string> = {
  workspace: "var(--primary)",
  project: "var(--muted-foreground)",
  agent: "var(--chart-4)",
  draft: "var(--primary)",
}

function miniMapColor(node: Node): string {
  return MINIMAP_COLORS[node.type ?? ""] ?? "var(--border)"
}

interface Pointer {
  clientX: number
  clientY: number
  preventDefault(): void
}

export function GraphSurface({ onDeleteNode }: { onDeleteNode(kind: NodeKind, id: string): void }) {
  const nodes = useGraphStore((state) => state.nodes)
  const edges = useGraphStore((state) => state.edges)
  const onNodesChange = useGraphStore((state) => state.onNodesChange)
  const onEdgesChange = useGraphStore((state) => state.onEdgesChange)
  const select = useGraphStore((state) => state.select)
  const persistPositions = useGraphStore((state) => state.persistPositions)
  const setHoveredNode = useGraphStore((state) => state.setHoveredNode)
  const setHoveredEdge = useGraphStore((state) => state.setHoveredEdge)
  const startDraft = useGraphStore((state) => state.startDraft)
  const linkAgent = useGraphStore((state) => state.linkAgent)

  const { screenToFlowPosition } = useReactFlow()
  const [menu, setMenu] = useState<GraphMenuState | null>(null)

  const openPaneMenu = (event: Pointer) => {
    event.preventDefault()
    setMenu({ x: event.clientX, y: event.clientY, target: null })
  }

  const handleConnect = (connection: Connection) => {
    const source = parseNodeId(connection.source)
    const target = parseNodeId(connection.target)
    if (source === null || target === null) {
      return
    }
    const agent = source.kind === "agent" ? source.id : target.kind === "agent" ? target.id : null
    const project =
      source.kind === "project" ? source.id : target.kind === "project" ? target.id : null
    if (agent === null || project === null) {
      return
    }
    linkAgent(agent, project)
  }

  const isValidConnection = (connection: Edge | Connection): boolean => {
    const source = parseNodeId(connection.source)
    const target = parseNodeId(connection.target)
    if (source === null || target === null) {
      return false
    }
    return (
      (source.kind === "agent" && target.kind === "project") ||
      (source.kind === "project" && target.kind === "agent")
    )
  }

  const openNodeMenu = (event: Pointer, node: Node) => {
    event.preventDefault()
    const parsed = parseNodeId(node.id)
    setMenu({
      x: event.clientX,
      y: event.clientY,
      target: parsed === null ? null : { kind: parsed.kind, id: parsed.id },
    })
  }

  const createDraft = (kind: DraftKind) => {
    if (menu === null) {
      return
    }
    startDraft(kind, screenToFlowPosition({ x: menu.x, y: menu.y }))
    setMenu(null)
  }

  return (
    <div className="h-full w-full" onContextMenu={(event) => event.preventDefault()}>
      <ReactFlow
        nodes={nodes}
        edges={edges}
        nodeTypes={nodeTypes}
        edgeTypes={edgeTypes}
        onNodesChange={onNodesChange}
        onEdgesChange={onEdgesChange}
        onNodeClick={(_event, node) => {
          const parsed = parseNodeId(node.id)
          select(parsed === null ? null : { kind: parsed.kind, id: parsed.id })
        }}
        onPaneClick={() => select(null)}
        onEdgeClick={() => select(null)}
        onNodeDragStop={() => persistPositions()}
        onNodeMouseEnter={(_event, node) => setHoveredNode(node.id)}
        onNodeMouseLeave={() => setHoveredNode(null)}
        onEdgeMouseEnter={(_event, edge: Edge) => setHoveredEdge(edge.id)}
        onEdgeMouseLeave={() => setHoveredEdge(null)}
        onPaneContextMenu={(event) => openPaneMenu(event)}
        onNodeContextMenu={(event, node) => openNodeMenu(event, node)}
        onConnect={handleConnect}
        isValidConnection={isValidConnection}
        colorMode="dark"
        fitView
        fitViewOptions={{ padding: 0.25 }}
        minZoom={0.3}
        maxZoom={1.5}
      >
        <Background variant={BackgroundVariant.Dots} gap={22} size={1} />
        <Controls showInteractive={false} />
        <MiniMap pannable zoomable nodeColor={miniMapColor} />
      </ReactFlow>

      {menu === null ? null : (
        <GraphContextMenu
          menu={menu}
          onCreate={createDraft}
          onDelete={onDeleteNode}
          onClose={() => setMenu(null)}
        />
      )}
    </div>
  )
}
