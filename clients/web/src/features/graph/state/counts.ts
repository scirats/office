import type { Edge } from "@xyflow/react"

export function deriveChildCounts(edges: Edge[]): Record<string, number> {
  const counts: Record<string, number> = {}
  for (const edge of edges) {
    counts[edge.source] = (counts[edge.source] ?? 0) + 1
  }
  return counts
}
