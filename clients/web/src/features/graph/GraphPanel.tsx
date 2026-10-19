import { useEffect, useMemo, useState } from "react"
import { ReactFlowProvider } from "@xyflow/react"

import { useOfficeStore } from "@/lib/office/store"

import { AgentModal } from "./agent/AgentModal"
import { DeleteNodeDialog, type DeleteNodeTarget } from "./DeleteNodeDialog"
import { GraphSurface } from "./GraphSurface"
import {
  agentNodeId,
  parseProjectKey,
  projectNodeId,
  workspaceNodeId,
  type NodeKind,
} from "./model"
import { Notice } from "./Notice"
import { ProjectModal } from "./ProjectModal"
import { computeGraphFocus } from "./state/focus"
import { GraphFocusProvider } from "./state/focusContext"
import { startGraphBridge } from "./state/graphBridge"
import { useGraphStore } from "./state/graphStore"
import { selectAgentDetail, selectProjectDetail, selectWorkspaceDetail } from "./state/selectors"
import { WorkspaceModal } from "./WorkspaceModal"

type DeleteSelection = { kind: NodeKind; id: string }

function nodeIdOf(selection: DeleteSelection): string {
  switch (selection.kind) {
    case "workspace":
      return workspaceNodeId(selection.id)
    case "project":
      return projectNodeId(selection.id)
    case "agent":
      return agentNodeId(selection.id)
  }
}

export function GraphPanel() {
  const connect = useOfficeStore((state) => state.connect)
  const notice = useOfficeStore((state) => state.notice)
  const clearNotice = useOfficeStore((state) => state.clearNotice)
  const deleteWorkspace = useOfficeStore((state) => state.deleteWorkspace)
  const deleteProject = useOfficeStore((state) => state.deleteProject)
  const deleteAgentDefinition = useOfficeStore((state) => state.deleteAgentDefinition)

  const nodes = useGraphStore((state) => state.nodes)
  const edges = useGraphStore((state) => state.edges)
  const selection = useGraphStore((state) => state.selection)
  const select = useGraphStore((state) => state.select)
  const hoveredNodeId = useGraphStore((state) => state.hoveredNodeId)
  const hoveredEdgeId = useGraphStore((state) => state.hoveredEdgeId)

  const [pendingDelete, setPendingDelete] = useState<DeleteSelection | null>(null)

  useEffect(() => {
    startGraphBridge()
    connect()
  }, [connect])

  const focus = useMemo(
    () => computeGraphFocus(edges, hoveredNodeId, hoveredEdgeId),
    [edges, hoveredNodeId, hoveredEdgeId],
  )

  const activeWorkspace = useMemo(
    () =>
      selection?.kind === "workspace" ? selectWorkspaceDetail(nodes, edges, selection.id) : null,
    [selection, nodes, edges],
  )
  const activeProject = useMemo(
    () => (selection?.kind === "project" ? selectProjectDetail(nodes, edges, selection.id) : null),
    [selection, nodes, edges],
  )
  const activeAgent = useMemo(
    () => (selection?.kind === "agent" ? selectAgentDetail(nodes, selection.id) : null),
    [selection, nodes],
  )

  const pendingNode = useMemo<DeleteNodeTarget | null>(() => {
    if (pendingDelete === null) {
      return null
    }
    const node = nodes.find((item) => item.id === nodeIdOf(pendingDelete))
    const name = (node?.data as { name?: string } | undefined)?.name
    return { ...pendingDelete, name: name ?? pendingDelete.id }
  }, [pendingDelete, nodes])

  const confirmDelete = (target: DeleteNodeTarget) => {
    switch (target.kind) {
      case "workspace":
        deleteWorkspace(target.id)
        break
      case "project": {
        const parsed = parseProjectKey(target.id)
        if (parsed !== null) {
          deleteProject(parsed.workspace, parsed.project)
        }
        break
      }
      case "agent":
        deleteAgentDefinition(target.id)
        break
    }
    setPendingDelete(null)
  }

  return (
    <ReactFlowProvider>
      <GraphFocusProvider value={focus}>
        <GraphSurface onDeleteNode={(kind, id) => setPendingDelete({ kind, id })} />
      </GraphFocusProvider>

      <WorkspaceModal workspace={activeWorkspace} onClose={() => select(null)} />
      <ProjectModal project={activeProject} onClose={() => select(null)} />
      <AgentModal agent={activeAgent} onClose={() => select(null)} />

      <DeleteNodeDialog
        target={pendingNode}
        onCancel={() => setPendingDelete(null)}
        onConfirm={confirmDelete}
      />

      <Notice message={notice} onDismiss={clearNotice} />
    </ReactFlowProvider>
  )
}
