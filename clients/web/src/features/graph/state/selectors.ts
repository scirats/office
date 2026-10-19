import type { Edge, Node } from "@xyflow/react"

import type { AgentDetail, AgentRef } from "../agent/types"
import {
  agentNodeId,
  projectNodeId,
  workspaceNodeId,
  type AgentNodeData,
  type ProjectNodeData,
  type WorkspaceNodeData,
} from "../model"
import type { ProjectDetail } from "../project"
import type { WorkspaceDetail } from "../workspace"

function agentsOf(nodes: Node[], edges: Edge[], sourceId: string): AgentRef[] {
  return edges.flatMap((edge) => {
    if (edge.source !== sourceId) {
      return []
    }
    const node = nodes.find((item) => item.id === edge.target)
    if (node === undefined || node.type !== "agent") {
      return []
    }
    const data = node.data as AgentNodeData
    return [{ id: node.id, name: data.name, role: data.role }]
  })
}

export function selectWorkspaceDetail(
  nodes: Node[],
  edges: Edge[],
  slug: string,
): WorkspaceDetail | null {
  const node = nodes.find((item) => item.id === workspaceNodeId(slug))
  if (node === undefined || node.type !== "workspace") {
    return null
  }
  const projects = edges.flatMap((edge) => {
    if (edge.source !== node.id) {
      return []
    }
    const project = nodes.find((item) => item.id === edge.target)
    if (project === undefined || project.type !== "project") {
      return []
    }
    return [
      {
        id: project.id,
        name: (project.data as ProjectNodeData).name,
        agents: agentsOf(nodes, edges, project.id),
      },
    ]
  })
  const data = node.data as WorkspaceNodeData
  return { id: node.id, name: data.name, user: data.user, home: data.home, projects }
}

export function selectProjectDetail(
  nodes: Node[],
  edges: Edge[],
  id: string,
): ProjectDetail | null {
  const node = nodes.find((item) => item.id === projectNodeId(id))
  if (node === undefined || node.type !== "project") {
    return null
  }
  return {
    id: node.id,
    name: (node.data as ProjectNodeData).name,
    agents: agentsOf(nodes, edges, node.id),
  }
}

export function selectAgentDetail(nodes: Node[], id: string): AgentDetail | null {
  const node = nodes.find((item) => item.id === agentNodeId(id))
  if (node === undefined || node.type !== "agent") {
    return null
  }
  const data = node.data as AgentNodeData
  return {
    id: node.id,
    definitionId: id,
    name: data.name,
    role: data.role,
    model: data.model,
  }
}
