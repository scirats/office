import { describe, expect, it } from "vitest"
import type { Edge } from "@xyflow/react"

import { computeGraphFocus, focusClassName } from "./focus"

const edges: Edge[] = [
  { id: "e1", source: "w", target: "p" },
  { id: "e2", source: "p", target: "a" },
]

describe("computeGraphFocus", () => {
  it("has no focus when nothing is hovered", () => {
    const focus = computeGraphFocus(edges, null, null)
    expect(focus.hasFocus).toBe(false)
    expect(focusClassName(focus, "w", "node")).toBe("")
  })

  it("activates a hovered node, its edges and direct neighbors", () => {
    const focus = computeGraphFocus(edges, "p", null)
    expect([...focus.activeNodeIds].sort()).toEqual(["a", "p", "w"])
    expect([...focus.activeEdgeIds].sort()).toEqual(["e1", "e2"])
    expect(focusClassName(focus, "p", "node")).toBe("is-active")
    expect(focusClassName(focus, "other", "node")).toBe("is-dim")
  })

  it("activates a hovered edge and its endpoints", () => {
    const focus = computeGraphFocus(edges, null, "e1")
    expect([...focus.activeNodeIds].sort()).toEqual(["p", "w"])
    expect([...focus.activeEdgeIds]).toEqual(["e1"])
    expect(focusClassName(focus, "e1", "edge")).toBe("is-active")
    expect(focusClassName(focus, "e2", "edge")).toBe("is-dim")
  })

  it("ignores a dangling edge id", () => {
    const focus = computeGraphFocus(edges, null, "missing")
    expect(focus.activeEdgeIds.size).toBe(0)
    expect(focus.activeNodeIds.size).toBe(0)
  })
})
