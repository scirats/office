import { useCallback, useEffect, useMemo, useRef, useState } from "react"
import {
  ReactFlowProvider,
  type Edge,
  type IsValidConnection,
  type OnConnect,
  type OnReconnect,
  type XYPosition,
} from "@xyflow/react"

import { OfficeClient } from "@/lib/office/client"
import {
  COLUMN_X,
  buildGraph,
  edgeId,
  isDraftNodeId,
  parseNodeId,
  workspaceNode,
  type AgentNodeData,
  type ProjectNodeData,
  type WorkspaceNodeData,
} from "./model"
import { AgentModal } from "./AgentModal"
import { GraphSurface, type GraphSurfaceHandlers } from "./GraphSurface"
import { ProjectModal } from "./ProjectModal"
import { WorkspaceModal } from "./WorkspaceModal"
import type { AgentDetail } from "./agent"
import type { ProjectDetail } from "./project"
import type { WorkspaceDetail } from "./workspace"
import { GraphActionsProvider, type GraphActions } from "./state/actions"
import { useGraphDrafts } from "./state/useGraphDrafts"
import { useGraphFocus } from "./state/useGraphFocus"
import { useGraphState } from "./state/useGraphState"

const initialModel = buildGraph()
const DAEMON_URL = import.meta.env.VITE_OFFICE_WS ?? "ws://127.0.0.1:7717"

export function GraphPanel() {
  const {
    nodes,
    edges,
    onNodesChange,
    onEdgesChange,
    connect,
    reconnect,
    disconnect,
    addNode,
    removeDraftNodes,
    getNodePosition,
    persistPositions,
  } = useGraphState(initialModel)

  const drafts = useGraphDrafts({
    addNode,
    removeDraftNodes,
    getNodePosition,
    persistPositions,
  })
  const [menuOpen, setMenuOpen] = useState(false)
  const [activeProjectId, setActiveProjectId] = useState<string | null>(null)
  const [activeWorkspaceId, setActiveWorkspaceId] = useState<string | null>(null)
  const [activeAgentId, setActiveAgentId] = useState<string | null>(null)
  const [notice, setNotice] = useState<string | null>(null)
  const modalOpen =
    activeProjectId !== null || activeWorkspaceId !== null || activeAgentId !== null
  const focus = useGraphFocus(nodes, edges, drafts.active || menuOpen || modalOpen)
  const clientRef = useRef<OfficeClient | null>(null)
  const workspacePositionRef = useRef<XYPosition | null>(null)
  const setHoveredNode = focus.setHoveredNode

  useEffect(() => {
    const client = new OfficeClient(DAEMON_URL)
    clientRef.current = client
    const unsubscribe = client.onEvent((event) => {
      if (event.type === "office.workspace.created.v1") {
        const position = workspacePositionRef.current ?? { x: COLUMN_X.workspace, y: 0 }
        workspacePositionRef.current = null
        const created = workspaceNode(
          {
            name: String(event.data.name ?? ""),
            slug: String(event.data.slug ?? ""),
            user: String(event.data.user ?? ""),
            home: String(event.data.home ?? ""),
          },
          position,
        )
        addNode(created)
        setHoveredNode(created.id)
      } else if (event.type === "office.workspace.listed.v1") {
        const workspaces = Array.isArray(event.data.workspaces) ? event.data.workspaces : []
        const inconsistent: string[] = []
        workspaces.forEach((entry, index) => {
          const record = entry as Record<string, unknown>
          if (String(record.status ?? "active") !== "active") {
            inconsistent.push(String(record.slug ?? ""))
          }
          addNode(
            workspaceNode(
              {
                name: String(record.name ?? ""),
                slug: String(record.slug ?? ""),
                user: String(record.user ?? ""),
                home: String(record.home ?? ""),
              },
              { x: COLUMN_X.workspace, y: 300 + index * 150 },
            ),
          )
        })
        if (inconsistent.length > 0) {
          setNotice(`inconsistent workspaces: ${inconsistent.join(", ")}`)
        }
      } else if (event.type === "office.workspace.status_changed.v1") {
        setNotice(`workspace ${String(event.data.slug ?? "")} is ${String(event.data.status ?? "")}`)
      } else if (event.type === "office.command.rejected.v1") {
        setNotice(String(event.data.message ?? "command rejected"))
      }
    })
    client.connect()
    return () => {
      unsubscribe()
      client.close()
      clientRef.current = null
    }
  }, [addNode, setHoveredNode])

  useEffect(() => {
    if (notice === null) {
      return
    }
    const timer = setTimeout(() => setNotice(null), 5000)
    return () => clearTimeout(timer)
  }, [notice])

  const actions = useMemo<GraphActions>(
    () => ({
      commitDraft: (input) => {
        focus.clear()
        if (drafts.kind === "workspace") {
          workspacePositionRef.current = drafts.position ?? { x: COLUMN_X.workspace, y: 0 }
          drafts.cancel()
          clientRef.current?.createWorkspace(input.name)
          return
        }
        const node = drafts.commit(input)
        if (node !== null) {
          focus.setHoveredNode(node.id)
        }
      },
      cancelDraft: () => {
        focus.clear()
        drafts.cancel()
      },
      removeEdge: disconnect,
    }),
    [drafts.kind, drafts.position, drafts.commit, drafts.cancel, focus, disconnect],
  )

  const isValidConnection = useCallback<IsValidConnection>(
    (connection) => {
      const source = parseNodeId(connection.source ?? "")
      const target = parseNodeId(connection.target ?? "")
      const allowed = source?.kind === "project" && target?.kind === "agent"
      if (!allowed) {
        return false
      }
      return !edges.some(
        (edge) => edge.id === edgeId(connection.source ?? "", connection.target ?? ""),
      )
    },
    [edges],
  )

  const onConnect = useCallback<OnConnect>(
    (connection) => connect(connection),
    [connect],
  )

  const onReconnect = useCallback<OnReconnect<Edge>>(
    (edge, connection) => reconnect(edge, connection),
    [reconnect],
  )

  const activeProject = useMemo<ProjectDetail | null>(() => {
    if (activeProjectId === null) {
      return null
    }
    const node = nodes.find((item) => item.id === activeProjectId)
    if (node === undefined || node.type !== "project") {
      return null
    }
    const agents = edges.flatMap((edge) => {
      if (edge.source !== activeProjectId) {
        return []
      }
      const target = nodes.find((item) => item.id === edge.target)
      if (target === undefined || target.type !== "agent") {
        return []
      }
      const data = target.data as AgentNodeData
      return [{ id: target.id, name: data.name, role: data.role }]
    })
    return { id: node.id, name: (node.data as ProjectNodeData).name, agents }
  }, [activeProjectId, nodes, edges])

  const activeWorkspace = useMemo<WorkspaceDetail | null>(() => {
    if (activeWorkspaceId === null) {
      return null
    }
    const node = nodes.find((item) => item.id === activeWorkspaceId)
    if (node === undefined || node.type !== "workspace") {
      return null
    }
    const projects = edges.flatMap((edge) => {
      if (edge.source !== activeWorkspaceId) {
        return []
      }
      const project = nodes.find((item) => item.id === edge.target)
      if (project === undefined || project.type !== "project") {
        return []
      }
      const agents = edges.flatMap((agentEdge) => {
        if (agentEdge.source !== project.id) {
          return []
        }
        const target = nodes.find((item) => item.id === agentEdge.target)
        if (target === undefined || target.type !== "agent") {
          return []
        }
        const data = target.data as AgentNodeData
        return [{ id: target.id, name: data.name, role: data.role }]
      })
      return [{ id: project.id, name: (project.data as ProjectNodeData).name, agents }]
    })
    const data = node.data as WorkspaceNodeData
    return {
      id: node.id,
      name: data.name,
      user: data.user,
      home: data.home,
      projects,
    }
  }, [activeWorkspaceId, nodes, edges])

  const activeAgent = useMemo<AgentDetail | null>(() => {
    if (activeAgentId === null) {
      return null
    }
    const node = nodes.find((item) => item.id === activeAgentId)
    if (node === undefined || node.type !== "agent") {
      return null
    }
    const projects = edges.flatMap((edge) => {
      if (edge.target !== activeAgentId) {
        return []
      }
      const source = nodes.find((item) => item.id === edge.source)
      if (source === undefined || source.type !== "project") {
        return []
      }
      return [{ id: source.id, name: (source.data as ProjectNodeData).name }]
    })
    const data = node.data as AgentNodeData
    return { id: node.id, name: data.name, role: data.role, projects }
  }, [activeAgentId, nodes, edges])

  const handlers = useMemo<GraphSurfaceHandlers>(
    () => ({
      onNodesChange,
      onEdgesChange,
      onConnect,
      onReconnect,
      isValidConnection,
      onNodeClick: (node) => {
        if (node.type === "project") {
          focus.clear()
          setActiveProjectId(node.id)
        } else if (node.type === "workspace") {
          focus.clear()
          setActiveWorkspaceId(node.id)
        } else if (node.type === "agent") {
          focus.clear()
          setActiveAgentId(node.id)
        }
      },
      onNodeDragStop: (node) => {
        if (!isDraftNodeId(node.id)) {
          persistPositions({ id: node.id, position: node.position })
        }
      },
      onNodeHover: focus.setHoveredNode,
      onEdgeHover: focus.setHoveredEdge,
      onMenuOpenChange: (open) => {
        setMenuOpen(open)
        focus.clear()
      },
      onCreateDraft: (kind, position) => {
        focus.clear()
        drafts.start(kind, position)
      },
    }),
    [
      onNodesChange,
      onEdgesChange,
      onConnect,
      onReconnect,
      isValidConnection,
      persistPositions,
      focus,
      drafts,
    ],
  )

  return (
    <ReactFlowProvider>
      <GraphActionsProvider value={actions}>
        <GraphSurface nodes={focus.nodes} edges={focus.edges} handlers={handlers} />
      </GraphActionsProvider>
      <ProjectModal project={activeProject} onClose={() => setActiveProjectId(null)} />
      <WorkspaceModal workspace={activeWorkspace} onClose={() => setActiveWorkspaceId(null)} />
      <AgentModal
        agent={activeAgent}
        onClose={() => setActiveAgentId(null)}
        onSaveDefinition={(definition) =>
          clientRef.current?.defineAgent({
            name: definition.name,
            role: definition.role,
            prompt: definition.prompt,
            tools: definition.tools.map((tool) => tool.name),
          })
        }
      />
      {notice === null ? null : (
        <div className="pointer-events-none fixed bottom-4 left-1/2 z-50 -translate-x-1/2 rounded-lg border border-destructive/40 bg-card px-3 py-1.5 text-xs text-destructive shadow-md">
          {notice}
        </div>
      )}
    </ReactFlowProvider>
  )
}
