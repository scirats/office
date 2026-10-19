import { z } from "zod"

export interface Workspace {
  name: string
  slug: string
  user: string
  home: string
  status: string
  createdAt: string
}

export interface ModelCapabilities {
  toolCalling: boolean
  vision: boolean
  reasoning: boolean
}

export interface LlmModel {
  id: string
  provider: string
  providerKind: string
  endpoint: string
  apiName: string
  displayName: string
  contextWindow: number
  maxOutputTokens: number
  capabilities: ModelCapabilities
  credentialConfigured: boolean
}

export interface Tool {
  id: string
  name: string
  description: string
  parameters: unknown
}

export interface AgentDefinition {
  id: string
  name: string
  role: string
  prompt: string
  model: string
  tools: string[]
  version: number
}

export interface AgentInstance {
  id: string
  agent: string
  workspace: string
  project: string
  createdAt: string
}

export interface Project {
  id: string
  workspace: string
  name: string
  createdAt: string
}

export interface TaskInput {
  company: string
  project: string
  agent: string
  input: string
}

export interface AgentDefinitionInput {
  name: string
  role: string
  prompt: string
  tools: string[]
  model: string
}

interface Envelope {
  subject?: string
  correlationId?: string
  sequence?: number
}

export type OfficeEvent =
  | ({ type: "command.accepted"; commandId: string } & Envelope)
  | ({ type: "command.rejected"; commandId: string; code: string; message: string } & Envelope)
  | ({ type: "office.error"; code: string; message: string } & Envelope)
  | ({ type: "workspace.created"; workspace: Workspace } & Envelope)
  | ({ type: "workspace.listed"; workspaces: Workspace[] } & Envelope)
  | ({ type: "workspace.deleted"; slug: string } & Envelope)
  | ({
      type: "workspace.status_changed"
      slug: string
      status: string
      inconsistencies: string[]
    } & Envelope)
  | ({ type: "project.created"; project: Project } & Envelope)
  | ({ type: "project.listed"; projects: Project[] } & Envelope)
  | ({ type: "project.deleted"; workspace: string; id: string } & Envelope)
  | ({ type: "model.listed"; models: LlmModel[] } & Envelope)
  | ({ type: "tool.listed"; tools: Tool[] } & Envelope)
  | ({ type: "secret.listed"; references: string[] } & Envelope)
  | ({ type: "secret.updated"; model: string } & Envelope)
  | ({ type: "agent.definition.created"; definition: AgentDefinition } & Envelope)
  | ({ type: "agent.definition.listed"; definitions: AgentDefinition[] } & Envelope)
  | ({ type: "agent.definition.deleted"; id: string } & Envelope)
  | ({ type: "agent.instance.created"; instance: AgentInstance } & Envelope)
  | ({ type: "agent.instance.listed"; instances: AgentInstance[] } & Envelope)
  | ({
      type: "agent.instance.deleted"
      agent: string
      workspace: string
      project: string
    } & Envelope)
  | ({ type: "task.accepted"; taskId: string; executionId: string } & Envelope)
  | ({
      type: "execution.state_changed"
      executionId: string
      previous: string
      current: string
    } & Envelope)
  | ({ type: "execution.log"; executionId: string; stream: string; line: string } & Envelope)

export type ParseResult =
  { kind: "event"; event: OfficeEvent } | { kind: "ignored" } | { kind: "invalid"; reason: string }

const envelopeSchema = z.object({
  id: z.string(),
  source: z.string(),
  type: z.string(),
  subject: z.string().optional(),
  correlationid: z.string().optional(),
  sequence: z.number().optional(),
  data: z.unknown(),
})

const workspaceSchema = z
  .object({
    name: z.string(),
    slug: z.string(),
    user: z.string(),
    home: z.string(),
    status: z.string().default("active"),
    created_at: z.string().default(""),
  })
  .transform((value): Workspace => ({
    name: value.name,
    slug: value.slug,
    user: value.user,
    home: value.home,
    status: value.status,
    createdAt: value.created_at,
  }))

const projectSchema = z
  .object({
    id: z.string(),
    workspace: z.string(),
    name: z.string(),
    created_at: z.string().default(""),
  })
  .transform((value): Project => ({
    id: value.id,
    workspace: value.workspace,
    name: value.name,
    createdAt: value.created_at,
  }))

const modelSchema = z
  .object({
    id: z.string(),
    provider: z.string(),
    provider_kind: z.string(),
    endpoint: z.string(),
    api_name: z.string(),
    display_name: z.string(),
    context_window: z.number(),
    max_output_tokens: z.number(),
    capabilities: z.object({
      tool_calling: z.boolean(),
      vision: z.boolean(),
      reasoning: z.boolean(),
    }),
    credential_configured: z.boolean(),
  })
  .transform((value): LlmModel => ({
    id: value.id,
    provider: value.provider,
    providerKind: value.provider_kind,
    endpoint: value.endpoint,
    apiName: value.api_name,
    displayName: value.display_name,
    contextWindow: value.context_window,
    maxOutputTokens: value.max_output_tokens,
    capabilities: {
      toolCalling: value.capabilities.tool_calling,
      vision: value.capabilities.vision,
      reasoning: value.capabilities.reasoning,
    },
    credentialConfigured: value.credential_configured,
  }))

const toolSchema = z
  .object({
    id: z.string(),
    name: z.string(),
    description: z.string(),
    parameters: z.unknown(),
  })
  .transform((value): Tool => ({
    id: value.id,
    name: value.name,
    description: value.description,
    parameters: value.parameters,
  }))

const agentDefinitionSchema = z
  .object({
    id: z.string(),
    name: z.string(),
    role: z.string(),
    prompt: z.string(),
    model: z.string(),
    tools: z.array(toolSchema),
    version: z.number(),
  })
  .transform((value): AgentDefinition => ({
    id: value.id,
    name: value.name,
    role: value.role,
    prompt: value.prompt,
    model: value.model,
    tools: value.tools.map((tool) => tool.id),
    version: value.version,
  }))

const agentInstanceSchema = z
  .object({
    id: z.string(),
    agent: z.string(),
    workspace: z.string(),
    project: z.string(),
    created_at: z.string().default(""),
  })
  .transform((value): AgentInstance => ({
    id: value.id,
    agent: value.agent,
    workspace: value.workspace,
    project: value.project,
    createdAt: value.created_at,
  }))

function invalid(type: string, error: z.ZodError): ParseResult {
  return { kind: "invalid", reason: `${type}: ${error.issues[0]?.message ?? "invalid payload"}` }
}

function parseData<T>(
  type: string,
  data: unknown,
  schema: z.ZodType<T>,
  build: (value: T, meta: Envelope) => OfficeEvent,
  meta: Envelope,
): ParseResult {
  const parsed = schema.safeParse(data)
  return parsed.success
    ? { kind: "event", event: build(parsed.data, meta) }
    : invalid(type, parsed.error)
}

export function parseOfficeEvent(raw: string): ParseResult {
  let json: unknown
  try {
    json = JSON.parse(raw)
  } catch {
    return { kind: "invalid", reason: "message is not valid JSON" }
  }
  const envelope = envelopeSchema.safeParse(json)
  if (!envelope.success) {
    return { kind: "invalid", reason: "malformed CloudEvent envelope" }
  }
  const { type, data, subject, correlationid, sequence } = envelope.data
  const meta: Envelope = { subject, correlationId: correlationid, sequence }

  switch (type) {
    case "office.command.accepted.v1":
      return parseData(
        type,
        data,
        z.object({ command_id: z.string() }),
        (value, m) => ({ type: "command.accepted", commandId: value.command_id, ...m }),
        meta,
      )
    case "office.command.rejected.v1":
      return parseData(
        type,
        data,
        z.object({ command_id: z.string(), code: z.string(), message: z.string() }),
        (value, m) => ({
          type: "command.rejected",
          commandId: value.command_id,
          code: value.code,
          message: value.message,
          ...m,
        }),
        meta,
      )
    case "office.error.v1":
      return parseData(
        type,
        data,
        z.object({ code: z.string(), message: z.string() }),
        (value, m) => ({ type: "office.error", code: value.code, message: value.message, ...m }),
        meta,
      )
    case "office.workspace.created.v1":
      return parseData(
        type,
        data,
        workspaceSchema,
        (workspace, m) => ({ type: "workspace.created", workspace, ...m }),
        meta,
      )
    case "office.workspace.listed.v1":
      return parseData(
        type,
        data,
        z.object({ workspaces: z.array(workspaceSchema) }),
        (value, m) => ({ type: "workspace.listed", workspaces: value.workspaces, ...m }),
        meta,
      )
    case "office.workspace.deleted.v1":
      return parseData(
        type,
        data,
        z.object({ slug: z.string() }),
        (value, m) => ({ type: "workspace.deleted", slug: value.slug, ...m }),
        meta,
      )
    case "office.workspace.status_changed.v1":
      return parseData(
        type,
        data,
        z.object({
          slug: z.string(),
          status: z.string(),
          inconsistencies: z.array(z.string()).default([]),
        }),
        (value, m) => ({
          type: "workspace.status_changed",
          slug: value.slug,
          status: value.status,
          inconsistencies: value.inconsistencies,
          ...m,
        }),
        meta,
      )
    case "office.project.created.v1":
      return parseData(
        type,
        data,
        projectSchema,
        (project, m) => ({ type: "project.created", project, ...m }),
        meta,
      )
    case "office.project.listed.v1":
      return parseData(
        type,
        data,
        z.object({ projects: z.array(projectSchema) }),
        (value, m) => ({ type: "project.listed", projects: value.projects, ...m }),
        meta,
      )
    case "office.project.deleted.v1":
      return parseData(
        type,
        data,
        z.object({ workspace: z.string(), id: z.string() }),
        (value, m) => ({
          type: "project.deleted",
          workspace: value.workspace,
          id: value.id,
          ...m,
        }),
        meta,
      )
    case "office.model.listed.v1":
      return parseData(
        type,
        data,
        z.object({ models: z.array(modelSchema) }),
        (value, m) => ({ type: "model.listed", models: value.models, ...m }),
        meta,
      )
    case "office.tool.listed.v1":
      return parseData(
        type,
        data,
        z.object({ tools: z.array(toolSchema) }),
        (value, m) => ({ type: "tool.listed", tools: value.tools, ...m }),
        meta,
      )
    case "office.secret.listed.v1":
      return parseData(
        type,
        data,
        z.object({ references: z.array(z.string()) }),
        (value, m) => ({ type: "secret.listed", references: value.references, ...m }),
        meta,
      )
    case "office.secret.updated.v1":
      return parseData(
        type,
        data,
        z.object({ model: z.string() }),
        (value, m) => ({ type: "secret.updated", model: value.model, ...m }),
        meta,
      )
    case "office.agent.definition.created.v1":
      return parseData(
        type,
        data,
        agentDefinitionSchema,
        (definition, m) => ({ type: "agent.definition.created", definition, ...m }),
        meta,
      )
    case "office.agent.definition.listed.v1":
      return parseData(
        type,
        data,
        z.object({ definitions: z.array(agentDefinitionSchema) }),
        (value, m) => ({ type: "agent.definition.listed", definitions: value.definitions, ...m }),
        meta,
      )
    case "office.agent.definition.deleted.v1":
      return parseData(
        type,
        data,
        z.object({ id: z.string() }),
        (value, m) => ({ type: "agent.definition.deleted", id: value.id, ...m }),
        meta,
      )
    case "office.agent.instance.created.v1":
      return parseData(
        type,
        data,
        agentInstanceSchema,
        (instance, m) => ({ type: "agent.instance.created", instance, ...m }),
        meta,
      )
    case "office.agent.instance.listed.v1":
      return parseData(
        type,
        data,
        z.object({ instances: z.array(agentInstanceSchema) }),
        (value, m) => ({ type: "agent.instance.listed", instances: value.instances, ...m }),
        meta,
      )
    case "office.agent.instance.deleted.v1":
      return parseData(
        type,
        data,
        z.object({ agent: z.string(), workspace: z.string(), project: z.string() }),
        (value, m) => ({
          type: "agent.instance.deleted",
          agent: value.agent,
          workspace: value.workspace,
          project: value.project,
          ...m,
        }),
        meta,
      )
    case "office.task.accepted.v1":
      return parseData(
        type,
        data,
        z.object({ task_id: z.string(), execution_id: z.string() }),
        (value, m) => ({
          type: "task.accepted",
          taskId: value.task_id,
          executionId: value.execution_id,
          ...m,
        }),
        meta,
      )
    case "office.execution.state_changed.v1":
      return parseData(
        type,
        data,
        z.object({ execution_id: z.string(), from: z.string(), to: z.string() }),
        (value, m) => ({
          type: "execution.state_changed",
          executionId: value.execution_id,
          previous: value.from,
          current: value.to,
          ...m,
        }),
        meta,
      )
    case "office.execution.log.v1":
      return parseData(
        type,
        data,
        z.object({ execution_id: z.string(), stream: z.string(), line: z.string() }),
        (value, m) => ({
          type: "execution.log",
          executionId: value.execution_id,
          stream: value.stream,
          line: value.line,
          ...m,
        }),
        meta,
      )
    default:
      return { kind: "ignored" }
  }
}

export interface WireCommand {
  type: string
  data: unknown
}

export const officeCommands = {
  subscribeAll: (): WireCommand => ({
    type: "office.subscription.add.v1",
    data: { filter: {} },
  }),
  listWorkspaces: (): WireCommand => ({ type: "office.workspace.list.v1", data: {} }),
  createWorkspace: (name: string): WireCommand => ({
    type: "office.workspace.create.v1",
    data: { name },
  }),
  deleteWorkspace: (slug: string): WireCommand => ({
    type: "office.workspace.delete.v1",
    data: { slug },
  }),
  listProjects: (): WireCommand => ({ type: "office.project.list.v1", data: {} }),
  createProject: (workspace: string, name: string): WireCommand => ({
    type: "office.project.create.v1",
    data: { workspace, name },
  }),
  deleteProject: (workspace: string, id: string): WireCommand => ({
    type: "office.project.delete.v1",
    data: { workspace, id },
  }),
  listModels: (): WireCommand => ({ type: "office.model.list.v1", data: {} }),
  listTools: (): WireCommand => ({ type: "office.tool.list.v1", data: {} }),
  listSecrets: (): WireCommand => ({ type: "office.secret.list.v1", data: {} }),
  setSecret: (model: string, value: string): WireCommand => ({
    type: "office.secret.set.v1",
    data: { model, value },
  }),
  listAgentDefinitions: (): WireCommand => ({
    type: "office.agent.definition.list.v1",
    data: {},
  }),
  defineAgent: (definition: AgentDefinitionInput): WireCommand => ({
    type: "office.agent.definition.create.v1",
    data: definition,
  }),
  deleteAgentDefinition: (id: string): WireCommand => ({
    type: "office.agent.definition.delete.v1",
    data: { id },
  }),
  listAgentInstances: (): WireCommand => ({
    type: "office.agent.instance.list.v1",
    data: {},
  }),
  createAgentInstance: (agent: string, workspace: string, project: string): WireCommand => ({
    type: "office.agent.instance.create.v1",
    data: { agent, workspace, project },
  }),
  deleteAgentInstance: (agent: string, workspace: string, project: string): WireCommand => ({
    type: "office.agent.instance.delete.v1",
    data: { agent, workspace, project },
  }),
  submitTask: (input: TaskInput): WireCommand => ({
    type: "office.task.submit.v1",
    data: input,
  }),
}
