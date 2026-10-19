import type {
  AgentDefinition,
  AgentInstance,
  LlmModel,
  OfficeEvent,
  Project,
  Tool,
  Workspace,
} from "./protocol"

export interface OfficeDomain {
  workspaces: Workspace[]
  projects: Project[]
  models: LlmModel[]
  tools: Tool[]
  secrets: string[]
  agentDefinitions: AgentDefinition[]
  agentInstances: AgentInstance[]
  notice: string | null
}

export const EMPTY_OFFICE_DOMAIN: OfficeDomain = {
  workspaces: [],
  projects: [],
  models: [],
  tools: [],
  secrets: [],
  agentDefinitions: [],
  agentInstances: [],
  notice: null,
}

function upsertById<T extends { id: string }>(items: T[], item: T): T[] {
  const index = items.findIndex((current) => current.id === item.id)
  if (index === -1) {
    return [...items, item]
  }
  const next = items.slice()
  next[index] = item
  return next
}

function upsertWorkspace(workspaces: Workspace[], workspace: Workspace): Workspace[] {
  const index = workspaces.findIndex((current) => current.slug === workspace.slug)
  if (index === -1) {
    return [...workspaces, workspace]
  }
  const next = workspaces.slice()
  next[index] = workspace
  return next
}

function upsertProject(projects: Project[], project: Project): Project[] {
  const index = projects.findIndex(
    (current) => current.workspace === project.workspace && current.id === project.id,
  )
  if (index === -1) {
    return [...projects, project]
  }
  const next = projects.slice()
  next[index] = project
  return next
}

export function reduceOfficeEvent(state: OfficeDomain, event: OfficeEvent): OfficeDomain {
  switch (event.type) {
    case "workspace.created":
      return { ...state, workspaces: upsertWorkspace(state.workspaces, event.workspace) }
    case "workspace.listed":
      return { ...state, workspaces: event.workspaces }
    case "workspace.deleted":
      return { ...state, workspaces: state.workspaces.filter((item) => item.slug !== event.slug) }
    case "workspace.status_changed":
      return {
        ...state,
        workspaces: state.workspaces.map((item) =>
          item.slug === event.slug ? { ...item, status: event.status } : item,
        ),
      }
    case "project.created":
      return { ...state, projects: upsertProject(state.projects, event.project) }
    case "project.listed":
      return { ...state, projects: event.projects }
    case "project.deleted":
      return {
        ...state,
        projects: state.projects.filter(
          (item) => !(item.workspace === event.workspace && item.id === event.id),
        ),
      }
    case "model.listed":
      return { ...state, models: event.models }
    case "tool.listed":
      return { ...state, tools: event.tools }
    case "secret.listed":
      return { ...state, secrets: event.references }
    case "secret.updated":
      return {
        ...state,
        models: state.models.map((item) =>
          item.id === event.model ? { ...item, credentialConfigured: true } : item,
        ),
      }
    case "agent.definition.created":
      return {
        ...state,
        agentDefinitions: upsertById(state.agentDefinitions, event.definition),
      }
    case "agent.definition.listed":
      return { ...state, agentDefinitions: event.definitions }
    case "agent.definition.deleted":
      return {
        ...state,
        agentDefinitions: state.agentDefinitions.filter((item) => item.id !== event.id),
        agentInstances: state.agentInstances.filter((item) => item.agent !== event.id),
      }
    case "agent.instance.created":
      return { ...state, agentInstances: upsertById(state.agentInstances, event.instance) }
    case "agent.instance.listed":
      return { ...state, agentInstances: event.instances }
    case "agent.instance.deleted":
      return {
        ...state,
        agentInstances: state.agentInstances.filter(
          (item) =>
            !(
              item.agent === event.agent &&
              item.workspace === event.workspace &&
              item.project === event.project
            ),
        ),
      }
    case "command.rejected":
    case "office.error":
      return { ...state, notice: event.message }
    default:
      return state
  }
}
