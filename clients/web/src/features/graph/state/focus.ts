import type { Edge } from "@xyflow/react"

export interface GraphFocus {
  activeNodeIds: Set<string>
  activeEdgeIds: Set<string>
  hasFocus: boolean
}

export const NO_FOCUS: GraphFocus = {
  activeNodeIds: new Set(),
  activeEdgeIds: new Set(),
  hasFocus: false,
}

export function computeGraphFocus(
  edges: Edge[],
  hoveredNodeId: string | null,
  hoveredEdgeId: string | null,
): GraphFocus {
  if (hoveredNodeId === null && hoveredEdgeId === null) {
    return NO_FOCUS
  }

  const activeNodeIds = new Set<string>()
  const activeEdgeIds = new Set<string>()

  if (hoveredNodeId !== null) {
    activeNodeIds.add(hoveredNodeId)
    for (const edge of edges) {
      if (edge.source === hoveredNodeId || edge.target === hoveredNodeId) {
        activeEdgeIds.add(edge.id)
        activeNodeIds.add(edge.source)
        activeNodeIds.add(edge.target)
      }
    }
  }

  if (hoveredEdgeId !== null) {
    const edge = edges.find((item) => item.id === hoveredEdgeId)
    if (edge !== undefined) {
      activeEdgeIds.add(edge.id)
      activeNodeIds.add(edge.source)
      activeNodeIds.add(edge.target)
    }
  }

  return { activeNodeIds, activeEdgeIds, hasFocus: true }
}

export function focusClassName(focus: GraphFocus, id: string, kind: "node" | "edge"): string {
  if (!focus.hasFocus) {
    return ""
  }
  const active = kind === "node" ? focus.activeNodeIds : focus.activeEdgeIds
  return active.has(id) ? "is-active" : "is-dim"
}
