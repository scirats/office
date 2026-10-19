import type { AgentRef } from "./agent/types"

export interface ProjectDetail {
  id: string
  name: string
  agents: AgentRef[]
}

export type TaskPriority = "low" | "medium" | "high"

export interface TaskCard {
  id: string
  title: string
  priority: TaskPriority
  agent: string
  execution: string
}

export interface TaskColumn {
  id: string
  title: string
  cards: TaskCard[]
}
