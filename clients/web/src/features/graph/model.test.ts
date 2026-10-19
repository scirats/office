import { describe, expect, it } from "vitest"

import {
  agentNodeId,
  assignmentEndpoints,
  draftNodeId,
  edgeId,
  isDraftNodeId,
  parseNodeId,
  parseProjectKey,
  projectKey,
  projectNodeId,
  workspaceNodeId,
} from "./model"

describe("node id codecs", () => {
  it("builds ids", () => {
    expect(workspaceNodeId("acme")).toBe("workspace:acme")
    expect(projectKey("acme", "ecommerce")).toBe("acme/ecommerce")
    expect(projectNodeId("acme/ecommerce")).toBe("project:acme/ecommerce")
    expect(agentNodeId("backend")).toBe("agent:backend")
    expect(edgeId("a", "b")).toBe("edge:a->b")
    expect(draftNodeId("agent")).toBe("draft:agent")
  })

  it("parses project keys", () => {
    expect(parseProjectKey("acme/ecommerce")).toEqual({
      workspace: "acme",
      project: "ecommerce",
    })
    expect(parseProjectKey("ecommerce")).toBeNull()
    expect(parseProjectKey("/ecommerce")).toBeNull()
    expect(parseProjectKey("acme/")).toBeNull()
  })

  it("parses known kinds and rejects drafts/unknowns", () => {
    expect(parseNodeId("agent:backend")).toEqual({ kind: "agent", id: "backend" })
    expect(parseNodeId("project:acme/ecommerce")).toEqual({
      kind: "project",
      id: "acme/ecommerce",
    })
    expect(parseNodeId("draft:agent")).toBeNull()
    expect(parseNodeId("nonsense")).toBeNull()
    expect(isDraftNodeId("draft:project")).toBe(true)
  })

  it("reads agent assignments from project edges only", () => {
    expect(
      assignmentEndpoints({
        source: projectNodeId("acme/ecommerce"),
        target: agentNodeId("backend"),
      }),
    ).toEqual({ agent: "backend", workspace: "acme", project: "ecommerce" })
    expect(
      assignmentEndpoints({
        source: workspaceNodeId("acme"),
        target: projectNodeId("acme/ecommerce"),
      }),
    ).toBeNull()
    expect(assignmentEndpoints({ source: "agent:a", target: "agent:b" })).toBeNull()
    expect(assignmentEndpoints({ source: "project:acme", target: "agent:a" })).toBeNull()
  })
})
