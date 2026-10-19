import { create } from "zustand"
import {
  applyEdgeChanges,
  applyNodeChanges,
  type Edge,
  type EdgeChange,
  type NodeChange,
  type XYPosition,
} from "@xyflow/react"

import type { AgentDefinition, AgentInstance, Project, Workspace } from "@/lib/office/protocol"
import { useOfficeStore } from "@/lib/office/store"

import {
  draftNode,
  isDraftNodeId,
  parseProjectKey,
  type DraftKind,
  type DraftSubmission,
  type NodeKind,
  type RenderedNode,
} from "../model"
import { deriveChildCounts } from "./counts"
import { loadGraphLayout, saveGraphLayout, type GraphLayout } from "./layout"
import { projectGraph } from "./projection"

export type GraphSelection = { kind: NodeKind; id: string } | null

interface DraftState {
  kind: DraftKind
  position: XYPosition
}

export interface ProjectionInput {
  workspaces: Workspace[]
  projects: Project[]
  agentDefinitions: AgentDefinition[]
  agentInstances: AgentInstance[]
}

export interface GraphStore {
  nodes: RenderedNode[]
  edges: Edge[]
  childCounts: Record<string, number>
  hoveredNodeId: string | null
  hoveredEdgeId: string | null
  selection: GraphSelection
  draft: DraftState | null

  applyProjection(input: ProjectionInput): void
  onNodesChange(changes: NodeChange<RenderedNode>[]): void
  onEdgesChange(changes: EdgeChange<Edge>[]): void
  select(selection: GraphSelection): void
  setHoveredNode(id: string | null): void
  setHoveredEdge(id: string | null): void
  startDraft(kind: DraftKind, position: XYPosition): void
  cancelDraft(): void
  commitDraft(submission: DraftSubmission): void
  linkAgent(agent: string, projectKey: string): void
  unlinkAgent(agent: string, projectKey: string): void
  persistPositions(): void
}

function buildLayout(nodes: RenderedNode[]): GraphLayout {
  const layout: GraphLayout = {}
  for (const node of nodes) {
    if (!isDraftNodeId(node.id)) {
      layout[node.id] = { x: node.position.x, y: node.position.y }
    }
  }
  return layout
}

export const useGraphStore = create<GraphStore>((set, get) => ({
  nodes: [],
  edges: [],
  childCounts: {},
  hoveredNodeId: null,
  hoveredEdgeId: null,
  selection: null,
  draft: null,

  applyProjection({ workspaces, projects, agentDefinitions, agentInstances }) {
    set((state) => {
      const layout = loadGraphLayout()
      const positions = new Map(state.nodes.map((node) => [node.id, node.position]))
      const projection = projectGraph(workspaces, projects, agentDefinitions, agentInstances)
      const projected: RenderedNode[] = projection.nodes.map((node) => ({
        ...node,
        position: positions.get(node.id) ?? layout[node.id] ?? node.position,
      }))
      const drafts = state.nodes.filter((node) => isDraftNodeId(node.id))
      return {
        nodes: [...projected, ...drafts],
        edges: projection.edges,
        childCounts: deriveChildCounts(projection.edges),
      }
    })
  },

  onNodesChange(changes) {
    set((state) => ({ nodes: applyNodeChanges(changes, state.nodes) }))
  },

  onEdgesChange(changes) {
    set((state) => {
      const edges = applyEdgeChanges(changes, state.edges)
      return { edges, childCounts: deriveChildCounts(edges) }
    })
  },

  select(selection) {
    set({ selection })
  },

  setHoveredNode(id) {
    set({ hoveredNodeId: id })
  },

  setHoveredEdge(id) {
    set({ hoveredEdgeId: id })
  },

  startDraft(kind, position) {
    set((state) => ({
      nodes: [...state.nodes.filter((node) => !isDraftNodeId(node.id)), draftNode(kind, position)],
      draft: { kind, position },
      selection: null,
    }))
  },

  cancelDraft() {
    set((state) => ({
      nodes: state.nodes.filter((node) => !isDraftNodeId(node.id)),
      draft: null,
    }))
  },

  commitDraft(submission) {
    const office = useOfficeStore.getState()
    switch (submission.kind) {
      case "workspace":
        office.createWorkspace(submission.name)
        break
      case "agent":
        office.defineAgent({
          name: submission.role,
          role: submission.role,
          prompt: "",
          tools: [],
          model: submission.model,
        })
        break
      case "project":
        office.createProject(submission.workspace, submission.project)
        break
    }
    get().cancelDraft()
  },

  linkAgent(agent, projectKey) {
    const parsed = parseProjectKey(projectKey)
    if (parsed === null) {
      return
    }
    const office = useOfficeStore.getState()
    const alreadyLinked = office.agentInstances.some(
      (instance) =>
        instance.agent === agent &&
        instance.workspace === parsed.workspace &&
        instance.project === parsed.project,
    )
    if (alreadyLinked) {
      return
    }
    office.createAgentInstance(agent, parsed.workspace, parsed.project)
  },

  unlinkAgent(agent, projectKey) {
    const parsed = parseProjectKey(projectKey)
    if (parsed === null) {
      return
    }
    useOfficeStore.getState().deleteAgentInstance(agent, parsed.workspace, parsed.project)
  },

  persistPositions() {
    saveGraphLayout(buildLayout(get().nodes))
  },
}))
