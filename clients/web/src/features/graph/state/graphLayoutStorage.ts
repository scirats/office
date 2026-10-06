import type { XYPosition } from "@xyflow/react"

export type GraphLayout = Record<string, XYPosition>

const STORAGE_KEY = "office.graph.layout"

export function loadGraphLayout(): GraphLayout {
  try {
    const raw = window.localStorage.getItem(STORAGE_KEY)
    return raw === null ? {} : (JSON.parse(raw) as GraphLayout)
  } catch (error) {
    console.error("office: could not read the saved graph layout", error)
    return {}
  }
}

export function saveGraphLayout(layout: GraphLayout): void {
  try {
    window.localStorage.setItem(STORAGE_KEY, JSON.stringify(layout))
  } catch (error) {
    console.error("office: could not persist the graph layout", error)
  }
}
