import { useOfficeStore } from "@/lib/office/store"

import { useGraphStore } from "./graphStore"

let started = false

export function startGraphBridge(): void {
  if (started) {
    return
  }
  started = true

  const sync = () => {
    const { workspaces, projects, agentDefinitions, agentInstances } = useOfficeStore.getState()
    useGraphStore
      .getState()
      .applyProjection({ workspaces, projects, agentDefinitions, agentInstances })
  }

  sync()
  let previous = useOfficeStore.getState()
  useOfficeStore.subscribe((state) => {
    if (
      state.workspaces !== previous.workspaces ||
      state.projects !== previous.projects ||
      state.agentDefinitions !== previous.agentDefinitions ||
      state.agentInstances !== previous.agentInstances
    ) {
      sync()
    }
    previous = state
  })
}
