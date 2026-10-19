import { z } from "zod"

const STORAGE_KEY = "office.graph.layout"

const layoutSchema = z.record(z.string(), z.object({ x: z.number(), y: z.number() }))

export type GraphLayout = z.infer<typeof layoutSchema>

export function loadGraphLayout(): GraphLayout {
  try {
    const raw = window.localStorage.getItem(STORAGE_KEY)
    if (raw === null) {
      return {}
    }
    const parsed = layoutSchema.safeParse(JSON.parse(raw))
    if (!parsed.success) {
      console.warn("[graph] ignoring invalid saved layout")
      return {}
    }
    return parsed.data
  } catch {
    return {}
  }
}

export function saveGraphLayout(layout: GraphLayout): void {
  try {
    window.localStorage.setItem(STORAGE_KEY, JSON.stringify(layout))
  } catch {}
}
