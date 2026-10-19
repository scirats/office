import type { Edge, Node, XYPosition } from "@xyflow/react"

export const GRAPH_COLUMNS = {
  workspace: 0,
  project: 360,
  agent: 720,
} as const

export const NODE_WIDTH = 256

export const DRAFT_KINDS = ["workspace", "project", "agent"] as const
export type DraftKind = (typeof DRAFT_KINDS)[number]

export const DRAFT_LABELS: Record<DraftKind, string> = {
  workspace: "New workspace",
  project: "New project",
  agent: "New agent",
}

export type WorkspaceNodeData = {
  name: string
  user: string
  home: string
}

export type ProjectNodeData = {
  name: string
}

export type AgentNodeData = {
  name: string
  role: string
  model: string
}

export type DraftNodeData = {
  kind: DraftKind
}

export type WorkspaceNode = Node<WorkspaceNodeData, "workspace">
export type ProjectNode = Node<ProjectNodeData, "project">
export type AgentNode = Node<AgentNodeData, "agent">
export type DraftNode = Node<DraftNodeData, "draft">
export type GraphNode = WorkspaceNode | ProjectNode | AgentNode
export type RenderedNode = GraphNode | DraftNode

export type NodeKind = "workspace" | "project" | "agent"

export const NODE_KIND_LABELS: Record<NodeKind, string> = {
  workspace: "Workspace",
  project: "Project",
  agent: "Agent",
}

export function workspaceNodeId(slug: string): string {
  return `workspace:${slug}`
}

export function projectKey(workspaceSlug: string, project: string): string {
  return `${workspaceSlug}/${project}`
}

export function parseProjectKey(key: string): { workspace: string; project: string } | null {
  const index = key.indexOf("/")
  if (index <= 0 || index === key.length - 1) {
    return null
  }
  return { workspace: key.slice(0, index), project: key.slice(index + 1) }
}

export function projectNodeId(key: string): string {
  return `project:${key}`
}

export function agentNodeId(id: string): string {
  return `agent:${id}`
}

export function draftNodeId(kind: DraftKind): string {
  return `draft:${kind}`
}

export function isDraftNodeId(id: string): boolean {
  return id.startsWith("draft:")
}

export function parseNodeId(value: string): { kind: NodeKind; id: string } | null {
  const index = value.indexOf(":")
  if (index === -1) {
    return null
  }
  const kind = value.slice(0, index)
  if (kind !== "workspace" && kind !== "project" && kind !== "agent") {
    return null
  }
  return { kind, id: value.slice(index + 1) }
}

export function edgeId(source: string, target: string): string {
  return `edge:${source}->${target}`
}

export function membershipEdge(workspaceId: string, projectId: string): Edge {
  return {
    id: edgeId(workspaceId, projectId),
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
    deletable: false,
    selectable: false,
    reconnectable: false,
  }
}

export interface AssignmentEndpoints {
  agent: string
  workspace: string
  project: string
}

export function assignmentEndpoints(edge: {
  source: string
  target: string
}): AssignmentEndpoints | null {
  const source = parseNodeId(edge.source)
  const target = parseNodeId(edge.target)
  if (source?.kind !== "project" || target?.kind !== "agent") {
    return null
  }
  const parsed = parseProjectKey(source.id)
  if (parsed === null) {
    return null
  }
  return { agent: target.id, workspace: parsed.workspace, project: parsed.project }
}

export type DraftSubmission =
  | { kind: "workspace"; name: string }
  | { kind: "agent"; role: string; model: string }
  | { kind: "project"; workspace: string; project: string }

export function draftNode(kind: DraftKind, position: XYPosition): DraftNode {
  return {
    id: draftNodeId(kind),
    type: "draft",
    position,
    data: { kind },
    selectable: false,
    deletable: false,
    connectable: false,
  }
}
