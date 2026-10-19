import { createContext, useContext } from "react"

import { NO_FOCUS, type GraphFocus } from "./focus"

const GraphFocusContext = createContext<GraphFocus>(NO_FOCUS)

export const GraphFocusProvider = GraphFocusContext.Provider

export function useGraphFocus(): GraphFocus {
  return useContext(GraphFocusContext)
}
