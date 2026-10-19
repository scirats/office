import { describe, expect, it } from "vitest"
import type { Edge, Node } from "@xyflow/react"

import {
  agentNodeId,
  assignmentEdge,
  membershipEdge,
  projectKey,
  projectNodeId,
  workspaceNodeId,
} from "../model"
import { selectAgentDetail, selectProjectDetail, selectWorkspaceDetail } from "./selectors"

const projectId = projectNodeId(projectKey("acme", "ecommerce"))

const nodes: Node[] = [
  {
    id: workspaceNodeId("acme"),
    type: "workspace",
    position: { x: 0, y: 0 },
    data: { name: "acme", user: "acme", home: "/home/acme" },
  },
  {
    id: projectId,
    type: "project",
    position: { x: 0, y: 0 },
    data: { name: "ecommerce" },
  },
  {
    id: agentNodeId("backend-engineer"),
    type: "agent",
    position: { x: 0, y: 0 },
    data: { name: "backend-engineer", role: "backend", model: "deepseek-flash" },
  },
]

const edges: Edge[] = [
  membershipEdge(workspaceNodeId("acme"), projectId),
  assignmentEdge(projectId, agentNodeId("backend-engineer")),
]

describe("graph selectors", () => {
  it("builds a workspace detail with nested projects and agents", () => {
    const detail = selectWorkspaceDetail(nodes, edges, "acme")
    expect(detail?.name).toBe("acme")
    expect(detail?.projects).toHaveLength(1)
    expect(detail?.projects[0].agents[0]).toEqual({
      id: agentNodeId("backend-engineer"),
      name: "backend-engineer",
      role: "backend",
    })
  })

  it("builds a project detail with its agents", () => {
    const detail = selectProjectDetail(nodes, edges, "acme/ecommerce")
    expect(detail?.name).toBe("ecommerce")
    expect(detail?.agents.map((agent) => agent.name)).toEqual(["backend-engineer"])
  })

  it("builds an agent detail with its model", () => {
    const detail = selectAgentDetail(nodes, "backend-engineer")
    expect(detail?.model).toBe("deepseek-flash")
  })

  it("returns null for unknown ids", () => {
    expect(selectWorkspaceDetail(nodes, edges, "ghost")).toBeNull()
    expect(selectProjectDetail(nodes, edges, "ghost/project")).toBeNull()
    expect(selectAgentDetail(nodes, "ghost")).toBeNull()
  })
})
