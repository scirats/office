import { describe, expect, it } from "vitest"
import type { Edge } from "@xyflow/react"

import { deriveChildCounts } from "./counts"

describe("deriveChildCounts", () => {
  it("counts outgoing edges per source", () => {
    const edges: Edge[] = [
      { id: "e1", source: "w", target: "p1" },
      { id: "e2", source: "w", target: "p2" },
      { id: "e3", source: "p1", target: "a1" },
    ]
    expect(deriveChildCounts(edges)).toEqual({ w: 2, p1: 1 })
  })

  it("returns an empty map for no edges", () => {
    expect(deriveChildCounts([])).toEqual({})
  })
})
