import { describe, expect, it } from "vitest"

import type { AgentDefinition, AgentInstance, Project, Workspace } from "@/lib/office/protocol"

import { projectGraph } from "./projection"

const workspace: Workspace = {
  name: "Acme",
  slug: "acme",
  user: "acme",
  home: "/home/acme",
  status: "active",
  createdAt: "",
}

const project: Project = {
  id: "ecommerce",
  workspace: "acme",
  name: "Ecommerce",
  createdAt: "",
}

const definition: AgentDefinition = {
  id: "backend-engineer",
  name: "Backend Engineer",
  role: "backend",
  prompt: "p",
  model: "deepseek-flash",
  tools: ["shell"],
  version: 1,
}

const instance: AgentInstance = {
  id: "inst_1",
  agent: "backend-engineer",
  workspace: "acme",
  project: "ecommerce",
  createdAt: "",
}

describe("projectGraph", () => {
  it("projects workspace, project and agent nodes with their links", () => {
    const { nodes, edges } = projectGraph([workspace], [project], [definition], [instance])
    expect(nodes.map((node) => node.type)).toEqual(["workspace", "project", "agent"])
    expect(edges.map((edge) => edge.id)).toEqual([
      "edge:workspace:acme->project:acme/ecommerce",
      "edge:project:acme/ecommerce->agent:backend-engineer",
    ])
  })

  it("draws a project without any agent", () => {
    const { nodes, edges } = projectGraph([workspace], [project], [definition], [])
    expect(nodes.map((node) => node.id)).toContain("project:acme/ecommerce")
    expect(edges.map((edge) => edge.id)).toEqual(["edge:workspace:acme->project:acme/ecommerce"])
  })

  it("links multiple agents to the same project", () => {
    const reviewer: AgentDefinition = {
      ...definition,
      id: "reviewer",
      name: "Reviewer",
      role: "reviewer",
    }
    const reviewerInstance: AgentInstance = { ...instance, id: "inst_2", agent: "reviewer" }
    const { edges } = projectGraph(
      [workspace],
      [project],
      [definition, reviewer],
      [instance, reviewerInstance],
    )
    expect(edges.map((edge) => edge.id)).toEqual([
      "edge:workspace:acme->project:acme/ecommerce",
      "edge:project:acme/ecommerce->agent:backend-engineer",
      "edge:project:acme/ecommerce->agent:reviewer",
    ])
  })

  it("separates projects with the same name in different workspaces", () => {
    const other: Workspace = { ...workspace, name: "Foo", slug: "foo" }
    const otherProject: Project = { ...project, workspace: "foo" }
    const { nodes } = projectGraph([workspace, other], [project, otherProject], [], [])
    const projects = nodes.filter((node) => node.type === "project")
    expect(projects.map((node) => node.id).sort()).toEqual([
      "project:acme/ecommerce",
      "project:foo/ecommerce",
    ])
  })

  it("skips links whose endpoints are missing", () => {
    const orphanInstance: AgentInstance = { ...instance, agent: "ghost" }
    const { edges } = projectGraph([workspace], [project], [definition], [orphanInstance])
    expect(edges.map((edge) => edge.id)).toEqual(["edge:workspace:acme->project:acme/ecommerce"])
  })

  it("is empty without daemon state", () => {
    expect(projectGraph([], [], [], [])).toEqual({ nodes: [], edges: [] })
  })
})
