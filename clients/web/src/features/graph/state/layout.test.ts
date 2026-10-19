import { beforeEach, describe, expect, it } from "vitest"

import { loadGraphLayout, saveGraphLayout } from "./layout"

beforeEach(() => {
  window.localStorage.clear()
})

describe("graph layout storage", () => {
  it("returns an empty layout when nothing is stored", () => {
    expect(loadGraphLayout()).toEqual({})
  })

  it("round-trips valid layouts", () => {
    saveGraphLayout({ "workspace:acme": { x: 10, y: 20 } })
    expect(loadGraphLayout()).toEqual({ "workspace:acme": { x: 10, y: 20 } })
  })

  it("ignores corrupted or invalid data", () => {
    window.localStorage.setItem("office.graph.layout", "{ not json")
    expect(loadGraphLayout()).toEqual({})
    window.localStorage.setItem("office.graph.layout", JSON.stringify({ node: { x: "nope" } }))
    expect(loadGraphLayout()).toEqual({})
  })
})
