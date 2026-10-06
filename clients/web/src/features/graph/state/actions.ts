import { createContext, useContext } from "react"

import type { DraftInput } from "../model"

export interface GraphActions {
  commitDraft(input: DraftInput): void
  cancelDraft(): void
  removeEdge(edgeId: string): void
}

const GraphActionsContext = createContext<GraphActions | null>(null)

export const GraphActionsProvider = GraphActionsContext.Provider

export function useGraphActions(): GraphActions {
  const value = useContext(GraphActionsContext)
  if (value === null) {
    throw new Error("useGraphActions must be used within a GraphActionsProvider")
  }
  return value
}
