import { describe, expect, it } from "vitest"

import type { LlmModel, OfficeEvent, Tool, Workspace } from "./protocol"
import { EMPTY_OFFICE_DOMAIN, reduceOfficeEvent } from "./reducer"

const workspace: Workspace = {
  name: "Acme",
  slug: "acme",
  user: "acme",
  home: "/home/acme",
  status: "active",
  createdAt: "2026-01-01",
}

const model: LlmModel = {
  id: "deepseek-flash",
  provider: "deepseek",
  providerKind: "deepseek",
  endpoint: "https://api.deepseek.com",
  apiName: "deepseek-flash",
  displayName: "DeepSeek V4.1 Flash",
  contextWindow: 1048576,
  maxOutputTokens: 384000,
  capabilities: { toolCalling: true, vision: true, reasoning: true },
  credentialConfigured: false,
}

const tool: Tool = { id: "shell", name: "shell", description: "run", parameters: {} }

function reduce(event: OfficeEvent) {
  return reduceOfficeEvent(EMPTY_OFFICE_DOMAIN, event)
}

describe("reduceOfficeEvent", () => {
  it("adds and updates workspaces without duplicates", () => {
    const added = reduce({ type: "workspace.created", workspace })
    expect(added.workspaces).toHaveLength(1)
    const updated = reduceOfficeEvent(added, {
      type: "workspace.created",
      workspace: { ...workspace, name: "Acme Corp" },
    })
    expect(updated.workspaces).toHaveLength(1)
    expect(updated.workspaces[0].name).toBe("Acme Corp")
  })

  it("replaces the list and removes on delete", () => {
    const listed = reduce({ type: "workspace.listed", workspaces: [workspace] })
    const removed = reduceOfficeEvent(listed, { type: "workspace.deleted", slug: "acme" })
    expect(removed.workspaces).toEqual([])
  })

  it("updates status and models", () => {
    const listed = reduce({ type: "workspace.listed", workspaces: [workspace] })
    const status = reduceOfficeEvent(listed, {
      type: "workspace.status_changed",
      slug: "acme",
      status: "inconsistent",
      inconsistencies: ["home"],
    })
    expect(status.workspaces[0].status).toBe("inconsistent")

    const withModel = reduceOfficeEvent(EMPTY_OFFICE_DOMAIN, {
      type: "model.listed",
      models: [model],
    })
    const configured = reduceOfficeEvent(withModel, {
      type: "secret.updated",
      model: "deepseek-flash",
    })
    expect(configured.models[0].credentialConfigured).toBe(true)
  })

  it("stores tools and surfaces errors as a notice", () => {
    expect(reduce({ type: "tool.listed", tools: [tool] }).tools).toEqual([tool])
    expect(
      reduce({ type: "command.rejected", commandId: "c", code: "command_failed", message: "boom" })
        .notice,
    ).toBe("boom")
    expect(reduce({ type: "office.error", code: "x", message: "nope" }).notice).toBe("nope")
  })

  it("stores agent definitions, instances and secret references", () => {
    const definition = {
      id: "backend-engineer",
      name: "Backend Engineer",
      role: "backend",
      prompt: "p",
      model: "deepseek-flash",
      tools: ["shell"],
      version: 1,
    }
    const instance = {
      id: "inst_1",
      agent: "backend-engineer",
      workspace: "acme",
      project: "ecommerce",
      createdAt: "",
    }

    const created = reduce({ type: "agent.definition.created", definition })
    expect(created.agentDefinitions).toEqual([definition])
    const bumped = reduceOfficeEvent(created, {
      type: "agent.definition.created",
      definition: { ...definition, version: 2 },
    })
    expect(bumped.agentDefinitions).toHaveLength(1)
    expect(bumped.agentDefinitions[0].version).toBe(2)
    expect(
      reduce({ type: "agent.definition.listed", definitions: [definition] }).agentDefinitions,
    ).toEqual([definition])

    const withInstance = reduce({ type: "agent.instance.created", instance })
    expect(withInstance.agentInstances).toEqual([instance])
    const removed = reduceOfficeEvent(withInstance, {
      type: "agent.instance.deleted",
      agent: "backend-engineer",
      workspace: "acme",
      project: "ecommerce",
    })
    expect(removed.agentInstances).toEqual([])
    expect(reduce({ type: "agent.instance.listed", instances: [instance] }).agentInstances).toEqual(
      [instance],
    )

    expect(reduce({ type: "secret.listed", references: ["provider/deepseek"] }).secrets).toEqual([
      "provider/deepseek",
    ])
  })

  it("removes an agent definition and its instances on delete", () => {
    const definition = {
      id: "backend-engineer",
      name: "Backend Engineer",
      role: "backend",
      prompt: "p",
      model: "deepseek-flash",
      tools: ["shell"],
      version: 1,
    }
    const instance = {
      id: "inst_1",
      agent: "backend-engineer",
      workspace: "acme",
      project: "ecommerce",
      createdAt: "",
    }
    const withInstance = reduceOfficeEvent(
      reduce({ type: "agent.definition.created", definition }),
      { type: "agent.instance.created", instance },
    )
    const removed = reduceOfficeEvent(withInstance, {
      type: "agent.definition.deleted",
      id: "backend-engineer",
    })
    expect(removed.agentDefinitions).toEqual([])
    expect(removed.agentInstances).toEqual([])
  })

  it("stores projects independently of agents", () => {
    const project = { id: "ecommerce", workspace: "acme", name: "Ecommerce", createdAt: "" }
    const created = reduce({ type: "project.created", project })
    expect(created.projects).toEqual([project])

    const bumped = reduceOfficeEvent(created, {
      type: "project.created",
      project: { ...project, name: "Ecommerce Platform" },
    })
    expect(bumped.projects).toHaveLength(1)
    expect(bumped.projects[0].name).toBe("Ecommerce Platform")

    const removed = reduceOfficeEvent(created, {
      type: "project.deleted",
      workspace: "acme",
      id: "ecommerce",
    })
    expect(removed.projects).toEqual([])
    expect(reduce({ type: "project.listed", projects: [project] }).projects).toEqual([project])
  })

  it("ignores events it does not model", () => {
    const unchanged = reduce({
      type: "execution.log",
      executionId: "e",
      stream: "stdout",
      line: "hi",
    })
    expect(unchanged).toBe(EMPTY_OFFICE_DOMAIN)
  })
})
