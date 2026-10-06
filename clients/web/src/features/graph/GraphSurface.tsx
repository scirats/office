import { useState } from "react"
import {
  Background,
  BackgroundVariant,
  ConnectionLineType,
  ConnectionMode,
  Controls,
  MiniMap,
  ReactFlow,
  useReactFlow,
  type Edge,
  type IsValidConnection,
  type Node,
  type OnConnect,
  type OnEdgesChange,
  type OnNodesChange,
  type OnReconnect,
  type XYPosition,
} from "@xyflow/react"
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuLabel,
  ContextMenuSeparator,
  ContextMenuTrigger,
} from "@/components/ui/context-menu"

import { DRAFT_KINDS, DRAFT_LABELS, type DraftKind } from "./model"
import { edgeTypes } from "./edgeTypes"
import { nodeTypes } from "./nodeTypes"
import { usePointerNodeFocus } from "./state/usePointerNodeFocus"

import "./graph.css"

export interface GraphSurfaceHandlers {
  onNodesChange: OnNodesChange<Node>
  onEdgesChange: OnEdgesChange<Edge>
  onConnect: OnConnect
  onReconnect: OnReconnect<Edge>
  isValidConnection: IsValidConnection
  onNodeClick: (node: Node) => void
  onNodeDragStop: (node: Node) => void
  onNodeHover: (id: string | null) => void
  onEdgeHover: (id: string | null) => void
  onMenuOpenChange: (open: boolean) => void
  onCreateDraft: (kind: DraftKind, position: XYPosition) => void
}

interface GraphSurfaceProps {
  nodes: Node[]
  edges: Edge[]
  handlers: GraphSurfaceHandlers
}

const MINIMAP_COLORS: Record<string, string> = {
  workspace: "var(--primary)",
  project: "var(--muted-foreground)",
  agent: "var(--chart-4)",
  draft: "var(--primary)",
}

function miniMapColor(node: Node): string {
  return MINIMAP_COLORS[node.type ?? ""] ?? "var(--border)"
}

export function GraphSurface({ nodes, edges, handlers }: GraphSurfaceProps) {
  const { screenToFlowPosition } = useReactFlow()
  const [flowPosition, setFlowPosition] = useState<XYPosition | null>(null)

  usePointerNodeFocus(handlers.onNodeHover)

  const openMenu = (event: { clientX: number; clientY: number }) => {
    setFlowPosition(screenToFlowPosition({ x: event.clientX, y: event.clientY }))
  }

  return (
    <ContextMenu
      modal={false}
      onOpenChange={(open) => {
        if (!open) {
          setFlowPosition(null)
        }
        handlers.onMenuOpenChange(open)
      }}
    >
      <ContextMenuTrigger asChild>
        <div className="h-full w-full">
          <ReactFlow
            nodes={nodes}
            edges={edges}
            nodeTypes={nodeTypes}
            edgeTypes={edgeTypes}
            onNodesChange={handlers.onNodesChange}
            onEdgesChange={handlers.onEdgesChange}
            onConnect={handlers.onConnect}
            onReconnect={handlers.onReconnect}
            edgesReconnectable
            isValidConnection={handlers.isValidConnection}
            onNodeClick={(_event, node) => handlers.onNodeClick(node)}
            onNodeDragStop={(_event, node) => handlers.onNodeDragStop(node)}
            onEdgeMouseEnter={(_event, edge) => handlers.onEdgeHover(edge.id)}
            onEdgeMouseLeave={() => handlers.onEdgeHover(null)}
            onPaneContextMenu={(event) => openMenu(event)}
            onNodeContextMenu={(event) => openMenu(event)}
            connectionMode={ConnectionMode.Strict}
            connectionLineType={ConnectionLineType.Bezier}
            connectionLineStyle={{
              stroke: "var(--primary)",
              strokeWidth: 1.5,
              strokeDasharray: "5 5",
            }}
            reconnectRadius={16}
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
        </div>
      </ContextMenuTrigger>
      {flowPosition === null ? null : (
        <ContextMenuContent onCloseAutoFocus={(event) => event.preventDefault()} className="w-48">
          <ContextMenuLabel>Create</ContextMenuLabel>
          <ContextMenuSeparator />
          {DRAFT_KINDS.map((kind) => (
            <ContextMenuItem
              key={kind}
              onSelect={() => handlers.onCreateDraft(kind, flowPosition)}
            >
              {DRAFT_LABELS[kind]}
            </ContextMenuItem>
          ))}
        </ContextMenuContent>
      )}
    </ContextMenu>
  )
}
