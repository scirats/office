import type { AgentRef } from "./agent/types"

export interface WorkspaceProject {
  id: string
  name: string
  agents: AgentRef[]
}

export interface WorkspaceDetail {
  id: string
  name: string
  user: string
  home: string
  projects: WorkspaceProject[]
}
