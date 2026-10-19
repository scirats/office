import type { Edge } from "@xyflow/react"

import type { AgentDefinition, AgentInstance, Project, Workspace } from "@/lib/office/protocol"

import {
  agentNodeId,
  assignmentEdge,
  GRAPH_COLUMNS,
  membershipEdge,
  projectKey,
  projectNodeId,
  workspaceNodeId,
  type GraphNode,
} from "../model"

const ROW_GAP = 140

export interface GraphProjection {
  nodes: GraphNode[]
  edges: Edge[]
}

export function projectGraph(
  workspaces: Workspace[],
  projects: Project[],
  definitions: AgentDefinition[],
  instances: AgentInstance[],
): GraphProjection {
  const nodes: GraphNode[] = []
  const nodeIds = new Set<string>()

  const push = (node: GraphNode): void => {
    if (nodeIds.has(node.id)) {
      return
    }
    nodeIds.add(node.id)
    nodes.push(node)
  }

  workspaces.forEach((workspace, index) => {
    push({
      id: workspaceNodeId(workspace.slug),
      type: "workspace",
      position: { x: GRAPH_COLUMNS.workspace, y: index * ROW_GAP },
      data: { name: workspace.name, user: workspace.user, home: workspace.home },
      deletable: false,
      connectable: false,
    })
  })

  projects.forEach((project, index) => {
    push({
      id: projectNodeId(projectKey(project.workspace, project.id)),
      type: "project",
      position: { x: GRAPH_COLUMNS.project, y: index * ROW_GAP },
      data: { name: project.name },
      deletable: false,
      connectable: true,
    })
  })

  definitions.forEach((definition, index) => {
    push({
      id: agentNodeId(definition.id),
      type: "agent",
      position: { x: GRAPH_COLUMNS.agent, y: index * ROW_GAP },
      data: { name: definition.name, role: definition.role, model: definition.model },
      deletable: false,
      connectable: true,
    })
  })

  const edges: Edge[] = []
  const edgeIds = new Set<string>()
  const link = (edge: Edge): void => {
    if (edgeIds.has(edge.id)) {
      return
    }
    edgeIds.add(edge.id)
    edges.push(edge)
  }

  for (const project of projects) {
    const workspaceId = workspaceNodeId(project.workspace)
    const id = projectNodeId(projectKey(project.workspace, project.id))
    if (nodeIds.has(workspaceId) && nodeIds.has(id)) {
      link(membershipEdge(workspaceId, id))
    }
  }

  for (const instance of instances) {
    const projectId = projectNodeId(projectKey(instance.workspace, instance.project))
    const agentId = agentNodeId(instance.agent)
    if (nodeIds.has(projectId) && nodeIds.has(agentId)) {
      link(assignmentEdge(projectId, agentId))
    }
  }

  return { nodes, edges }
}
