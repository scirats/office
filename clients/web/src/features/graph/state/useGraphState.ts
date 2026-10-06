import { useCallback, useEffect, useMemo, useRef } from "react"
import {
  reconnectEdge as reconnectFlowEdge,
  useEdgesState,
  useNodesState,
  type Connection,
  type Edge,
  type Node,
  type OnEdgesChange,
  type OnNodesChange,
  type XYPosition,
} from "@xyflow/react"

import {
  assignmentEdge,
  edgeId,
  isDraftNodeId,
  type GraphModel,
  type GraphNode,
} from "../model"
import { loadGraphLayout, saveGraphLayout, type GraphLayout } from "./graphLayoutStorage"

export interface GraphState {
  nodes: Node[]
  edges: Edge[]
  onNodesChange: OnNodesChange<Node>
  onEdgesChange: OnEdgesChange<Edge>
  connect: (connection: Connection) => void
  reconnect: (edge: Edge, connection: Connection) => void
  disconnect: (edgeId: string) => void
  addNode: (node: Node) => void
  removeDraftNodes: () => void
  getNodePosition: (id: string) => XYPosition | undefined
  persistPositions: (extra?: { id: string; position: XYPosition }) => void
}

function withSavedPositions(nodes: GraphNode[], layout: GraphLayout): Node[] {
  return nodes.map((node) => {
    const position = layout[node.id]
    return position === undefined ? node : { ...node, position }
  })
}

export function useGraphState(model: GraphModel): GraphState {
  const saved = useRef<GraphLayout | null>(null)
  if (saved.current === null) {
    saved.current = loadGraphLayout()
  }

  const [nodes, setNodes, onNodesChange] = useNodesState<Node>(
    withSavedPositions(model.nodes, saved.current),
  )
  const [edges, setEdges, onEdgesChange] = useEdgesState<Edge>(model.edges)

  const nodesRef = useRef(nodes)
  useEffect(() => {
    nodesRef.current = nodes
  }, [nodes])

  const connect = useCallback(
    (connection: Connection) => {
      const { source, target } = connection
      if (source === null || target === null) {
        return
      }
      setEdges((current) => {
        const edge = assignmentEdge(source, target)
        return current.some((item) => item.id === edge.id) ? current : [...current, edge]
      })
    },
    [setEdges],
  )

  const reconnect = useCallback(
    (edge: Edge, connection: Connection) => {
      const { source, target } = connection
      if (source === null || target === null) {
        return
      }
      setEdges((current) =>
        reconnectFlowEdge(edge, connection, current).map((item) =>
          item.id === edge.id ? { ...item, id: edgeId(source, target) } : item,
        ),
      )
    },
    [setEdges],
  )

  const disconnect = useCallback(
    (targetEdgeId: string) => {
      setEdges((current) => current.filter((edge) => edge.id !== targetEdgeId))
    },
    [setEdges],
  )

  const addNode = useCallback(
    (node: Node) => {
      setNodes((current) =>
        current.some((item) => item.id === node.id) ? current : [...current, node],
      )
    },
    [setNodes],
  )

  const removeDraftNodes = useCallback(() => {
    setNodes((current) => current.filter((node) => !isDraftNodeId(node.id)))
  }, [setNodes])

  const getNodePosition = useCallback(
    (id: string) => nodesRef.current.find((node) => node.id === id)?.position,
    [],
  )

  const persistPositions = useCallback(
    (extra?: { id: string; position: XYPosition }) => {
      const layout: GraphLayout = {}
      for (const node of nodesRef.current) {
        if (isDraftNodeId(node.id)) {
          continue
        }
        layout[node.id] = { x: node.position.x, y: node.position.y }
      }
      if (extra !== undefined) {
        layout[extra.id] = { x: extra.position.x, y: extra.position.y }
      }
      saved.current = layout
      saveGraphLayout(layout)
    },
    [],
  )

  return useMemo<GraphState>(
    () => ({
      nodes,
      edges,
      onNodesChange,
      onEdgesChange,
      connect,
      reconnect,
      disconnect,
      addNode,
      removeDraftNodes,
      getNodePosition,
      persistPositions,
    }),
    [
      nodes,
      edges,
      onNodesChange,
      onEdgesChange,
      connect,
      reconnect,
      disconnect,
      addNode,
      removeDraftNodes,
      getNodePosition,
      persistPositions,
    ],
  )
}
