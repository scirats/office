import type { Edge, Node, XYPosition } from "@xyflow/react"

export const COLUMN_X = {
  workspace: 0,
  project: 360,
  agent: 720,
} as const

export const NODE_WIDTH = 256

const DRAFT_PREFIX = "draft:"

export const DRAFT_KINDS = ["workspace", "project", "agent"] as const
export type DraftKind = (typeof DRAFT_KINDS)[number]

export const DRAFT_LABELS: Record<DraftKind, string> = {
  workspace: "New workspace",
  project: "New project",
  agent: "New agent",
}

export const DRAFT_OPTIONS: Record<DraftKind, string[]> = {
  workspace: [],
  project: [],
  agent: ["backend", "frontend", "security", "reviewer", "architect"],
}

export interface DraftInput {
  name: string
  detail: string
}

export type LocalDraftKind = Exclude<DraftKind, "workspace">

export type WorkspaceNodeData = {
  name: string
  projectCount: number
  user: string
  home: string
}

export type ProjectNodeData = {
  name: string
  agentCount: number
}

export type AgentNodeData = {
  name: string
  role: string
}

export type DraftNodeData = {
  kind: DraftKind
}

export type WorkspaceNode = Node<WorkspaceNodeData, "workspace">
export type ProjectNode = Node<ProjectNodeData, "project">
export type AgentNode = Node<AgentNodeData, "agent">
export type DraftNode = Node<DraftNodeData, "draft">
export type GraphNode = WorkspaceNode | ProjectNode | AgentNode

export interface GraphModel {
  nodes: GraphNode[]
  edges: Edge[]
}

export function workspaceNodeId(id: string): string {
  return `workspace:${id}`
}

export function projectNodeId(id: string): string {
  return `project:${id}`
}

export function agentNodeId(id: string): string {
  return `agent:${id}`
}

export function draftNodeId(kind: DraftKind): string {
  return `${DRAFT_PREFIX}${kind}`
}

export function isDraftNodeId(id: string): boolean {
  return id.startsWith(DRAFT_PREFIX)
}

export function parseNodeId(value: string): { kind: string; id: string } | null {
  const index = value.indexOf(":")
  if (index === -1) {
    return null
  }
  return { kind: value.slice(0, index), id: value.slice(index + 1) }
}

export function edgeId(source: string, target: string): string {
  return `edge:${source}->${target}`
}

function membershipEdge(workspaceId: string, projectId: string): Edge {
  return {
    id: `member:${projectId}`,
    source: workspaceId,
    target: projectId,
    type: "graph",
    className: "graph-edge",
    animated: true,
    deletable: false,
    selectable: false,
    reconnectable: false,
  }
}

export function assignmentEdge(projectId: string, agentId: string): Edge {
  return {
    id: edgeId(projectId, agentId),
    source: projectId,
    target: agentId,
    type: "graph",
    className: "graph-edge",
    animated: true,
    deletable: true,
    reconnectable: true,
  }
}

export function buildGraph(): GraphModel {
  const workspaceId = workspaceNodeId("acme")
  const ecommerceId = projectNodeId("ecommerce")
  const mobileId = projectNodeId("mobile")

  const nodes: GraphNode[] = [
    {
      id: workspaceId,
      type: "workspace",
      position: { x: COLUMN_X.workspace, y: 120 },
      data: { name: "acme", projectCount: 2, user: "acme", home: "/home/acme" },
      deletable: false,
      connectable: false,
    },
    {
      id: ecommerceId,
      type: "project",
      position: { x: COLUMN_X.project, y: 0 },
      data: { name: "ecommerce", agentCount: 2 },
      deletable: false,
      connectable: true,
    },
    {
      id: mobileId,
      type: "project",
      position: { x: COLUMN_X.project, y: 240 },
      data: { name: "mobile", agentCount: 2 },
      deletable: false,
      connectable: true,
    },
    {
      id: agentNodeId("backend-engineer"),
      type: "agent",
      position: { x: COLUMN_X.agent, y: -40 },
      data: { name: "backend-engineer", role: "backend" },
      deletable: false,
      connectable: true,
    },
    {
      id: agentNodeId("frontend-engineer"),
      type: "agent",
      position: { x: COLUMN_X.agent, y: 140 },
      data: { name: "frontend-engineer", role: "frontend" },
      deletable: false,
      connectable: true,
    },
    {
      id: agentNodeId("security-engineer"),
      type: "agent",
      position: { x: COLUMN_X.agent, y: 320 },
      data: { name: "security-engineer", role: "security" },
      deletable: false,
      connectable: true,
    },
  ]

  const edges: Edge[] = [
    membershipEdge(workspaceId, ecommerceId),
    membershipEdge(workspaceId, mobileId),
    assignmentEdge(ecommerceId, agentNodeId("backend-engineer")),
    assignmentEdge(ecommerceId, agentNodeId("frontend-engineer")),
    assignmentEdge(mobileId, agentNodeId("backend-engineer")),
    assignmentEdge(mobileId, agentNodeId("security-engineer")),
  ]

  return { nodes, edges }
}

export function createNode(
  kind: LocalDraftKind,
  id: string,
  input: DraftInput,
  position: XYPosition,
): GraphNode {
  switch (kind) {
    case "project":
      return {
        id: projectNodeId(id),
        type: "project",
        position,
        data: { name: input.name, agentCount: 0 },
        connectable: true,
      }
    case "agent":
      return {
        id: agentNodeId(id),
        type: "agent",
        position,
        data: { name: input.name, role: input.detail || DRAFT_OPTIONS.agent[0] },
        connectable: true,
      }
  }
}

export type ProvisionedWorkspace = {
  name: string
  slug: string
  user: string
  home: string
}

export function workspaceNode(
  workspace: ProvisionedWorkspace,
  position: XYPosition,
): WorkspaceNode {
  return {
    id: workspaceNodeId(workspace.slug),
    type: "workspace",
    position,
    data: {
      name: workspace.name,
      projectCount: 0,
      user: workspace.user,
      home: workspace.home,
    },
    deletable: false,
    connectable: false,
  }
}
