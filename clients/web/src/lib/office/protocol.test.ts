import { describe, expect, it } from "vitest"

import { officeCommands, parseOfficeEvent } from "./protocol"

function frame(type: string, data: unknown, extra: Record<string, unknown> = {}): string {
  return JSON.stringify({
    specversion: "1.0",
    id: "evt-1",
    source: "/office/daemon",
    type,
    data,
    ...extra,
  })
}

describe("parseOfficeEvent", () => {
  it("rejects non-JSON payloads", () => {
    expect(parseOfficeEvent("not json")).toEqual({
      kind: "invalid",
      reason: "message is not valid JSON",
    })
  })

  it("rejects a malformed envelope", () => {
    expect(parseOfficeEvent(JSON.stringify({ type: "x" })).kind).toBe("invalid")
  })

  it("ignores unknown event types", () => {
    expect(parseOfficeEvent(frame("office.nope.v1", {})).kind).toBe("ignored")
  })

  it("rejects a known type with invalid data", () => {
    expect(parseOfficeEvent(frame("office.workspace.created.v1", { name: 1 })).kind).toBe("invalid")
  })

  it("normalizes workspace.created", () => {
    const result = parseOfficeEvent(
      frame(
        "office.workspace.created.v1",
        { name: "Acme", slug: "acme", user: "acme", home: "/home/acme" },
        { subject: "workspace:acme", correlationid: "cmd-1" },
      ),
    )
    expect(result).toEqual({
      kind: "event",
      event: {
        type: "workspace.created",
        subject: "workspace:acme",
        correlationId: "cmd-1",
        sequence: undefined,
        workspace: {
          name: "Acme",
          slug: "acme",
          user: "acme",
          home: "/home/acme",
          status: "active",
          createdAt: "",
        },
      },
    })
  })

  it("normalizes model.listed snake_case into camelCase", () => {
    const result = parseOfficeEvent(
      frame("office.model.listed.v1", {
        models: [
          {
            id: "deepseek-flash",
            provider: "deepseek",
            provider_kind: "deepseek",
            endpoint: "https://api.deepseek.com",
            api_name: "deepseek-flash",
            display_name: "DeepSeek V4.1 Flash",
            context_window: 1048576,
            max_output_tokens: 384000,
            capabilities: { tool_calling: true, vision: true, reasoning: true },
            credential_configured: false,
          },
        ],
      }),
    )
    if (result.kind !== "event" || result.event.type !== "model.listed") {
      throw new Error("expected model.listed")
    }
    expect(result.event.models[0]).toMatchObject({
      id: "deepseek-flash",
      providerKind: "deepseek",
      contextWindow: 1048576,
      credentialConfigured: false,
      capabilities: { toolCalling: true, vision: true, reasoning: true },
    })
  })

  it("normalizes agent definitions resolving tools to ids", () => {
    const result = parseOfficeEvent(
      frame("office.agent.definition.listed.v1", {
        definitions: [
          {
            id: "backend-engineer",
            name: "Backend Engineer",
            role: "backend",
            prompt: "p",
            model: "deepseek-flash",
            tools: [{ id: "shell", name: "shell", description: "run", parameters: {} }],
            version: 1,
          },
        ],
      }),
    )
    if (result.kind !== "event" || result.event.type !== "agent.definition.listed") {
      throw new Error("expected agent.definition.listed")
    }
    expect(result.event.definitions[0]).toMatchObject({
      id: "backend-engineer",
      model: "deepseek-flash",
      tools: ["shell"],
      version: 1,
    })
  })

  it("normalizes agent instances and secret references", () => {
    const instances = parseOfficeEvent(
      frame("office.agent.instance.listed.v1", {
        instances: [
          { id: "inst_1", agent: "backend-engineer", workspace: "acme", project: "ecommerce" },
        ],
      }),
    )
    if (instances.kind !== "event" || instances.event.type !== "agent.instance.listed") {
      throw new Error("expected agent.instance.listed")
    }
    expect(instances.event.instances[0]).toEqual({
      id: "inst_1",
      agent: "backend-engineer",
      workspace: "acme",
      project: "ecommerce",
      createdAt: "",
    })

    const secrets = parseOfficeEvent(
      frame("office.secret.listed.v1", { references: ["provider/deepseek"] }),
    )
    if (secrets.kind !== "event" || secrets.event.type !== "secret.listed") {
      throw new Error("expected secret.listed")
    }
    expect(secrets.event.references).toEqual(["provider/deepseek"])
  })

  it("normalizes agent.definition.deleted", () => {
    const result = parseOfficeEvent(
      frame("office.agent.definition.deleted.v1", { id: "backend-engineer" }),
    )
    expect(result).toEqual({
      kind: "event",
      event: {
        type: "agent.definition.deleted",
        id: "backend-engineer",
        subject: undefined,
        correlationId: undefined,
        sequence: undefined,
      },
    })
  })

  it("normalizes execution.state_changed and keeps correlation", () => {
    const result = parseOfficeEvent(
      frame("office.execution.state_changed.v1", {
        execution_id: "exec-1",
        from: "READY",
        to: "RUNNING",
      }),
    )
    expect(result).toEqual({
      kind: "event",
      event: {
        type: "execution.state_changed",
        executionId: "exec-1",
        previous: "READY",
        current: "RUNNING",
        subject: undefined,
        correlationId: undefined,
        sequence: undefined,
      },
    })
  })
})

describe("officeCommands", () => {
  it("builds the wire type and payload", () => {
    expect(officeCommands.createWorkspace("Acme")).toEqual({
      type: "office.workspace.create.v1",
      data: { name: "Acme" },
    })
    expect(officeCommands.setSecret("deepseek-flash", "sk-1")).toEqual({
      type: "office.secret.set.v1",
      data: { model: "deepseek-flash", value: "sk-1" },
    })
    expect(
      officeCommands.submitTask({ company: "", project: "", agent: "sre", input: "x" }),
    ).toEqual({
      type: "office.task.submit.v1",
      data: { company: "", project: "", agent: "sre", input: "x" },
    })
    expect(officeCommands.deleteAgentDefinition("backend-engineer")).toEqual({
      type: "office.agent.definition.delete.v1",
      data: { id: "backend-engineer" },
    })
  })
})
