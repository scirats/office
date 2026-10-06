import { useCallback, useMemo, useState } from "react"
import type { Edge, Node } from "@xyflow/react"

import { cn } from "@/lib/utils"

export interface GraphFocus {
  nodes: Node[]
  edges: Edge[]
  setHoveredNode: (id: string | null) => void
  setHoveredEdge: (id: string | null) => void
  clear: () => void
}

export function useGraphFocus(
  sourceNodes: Node[],
  sourceEdges: Edge[],
  suspended: boolean,
): GraphFocus {
  const [hoveredNode, setHoveredNode] = useState<string | null>(null)
  const [hoveredEdge, setHoveredEdge] = useState<string | null>(null)

  const clear = useCallback(() => {
    setHoveredNode(null)
    setHoveredEdge(null)
  }, [])

  const focusIds = useMemo(() => {
    const ids = new Set<string>()
    if (hoveredEdge !== null) {
      const edge = sourceEdges.find((item) => item.id === hoveredEdge)
      if (edge !== undefined) {
        ids.add(edge.source)
        ids.add(edge.target)
      }
    } else if (hoveredNode !== null && sourceNodes.some((node) => node.id === hoveredNode)) {
      ids.add(hoveredNode)
      for (const edge of sourceEdges) {
        if (edge.source === hoveredNode) {
          ids.add(edge.target)
        }
        if (edge.target === hoveredNode) {
          ids.add(edge.source)
        }
      }
    }
    return ids
  }, [hoveredEdge, hoveredNode, sourceEdges, sourceNodes])

  const focusing = !suspended && focusIds.size > 0

  const nodes = useMemo<Node[]>(() => {
    if (!focusing) {
      return sourceNodes
    }
    return sourceNodes.map((node) => ({
      ...node,
      className: focusIds.has(node.id) || node.type === "draft" ? "is-active" : "is-dim",
    }))
  }, [sourceNodes, focusing, focusIds])

  const edges = useMemo<Edge[]>(() => {
    if (!focusing) {
      return sourceEdges
    }
    return sourceEdges.map((edge) => {
      const active =
        hoveredEdge === edge.id ||
        (hoveredNode !== null && (edge.source === hoveredNode || edge.target === hoveredNode))
      return { ...edge, className: cn(edge.className, active ? "is-active" : "is-dim") }
    })
  }, [sourceEdges, focusing, hoveredEdge, hoveredNode])

  return useMemo(
    () => ({ nodes, edges, setHoveredNode, setHoveredEdge, clear }),
    [nodes, edges, clear],
  )
}
