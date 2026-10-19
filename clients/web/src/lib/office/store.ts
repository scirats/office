import { create } from "zustand"

import { OfficeClient, type ConnectionState } from "./client"
import {
  officeCommands,
  type AgentDefinitionInput,
  type OfficeEvent,
  type TaskInput,
} from "./protocol"
import { EMPTY_OFFICE_DOMAIN, reduceOfficeEvent, type OfficeDomain } from "./reducer"

const DEFAULT_URL = "ws://127.0.0.1:7717"

const client = new OfficeClient({
  url: import.meta.env.VITE_OFFICE_WS ?? DEFAULT_URL,
})

export function subscribeOfficeEvents(listener: (event: OfficeEvent) => void): () => void {
  return client.onEvent(listener)
}

interface OfficeStore extends OfficeDomain {
  connection: ConnectionState
  connect(): void
  createWorkspace(name: string): void
  deleteWorkspace(slug: string): void
  createProject(workspace: string, project: string): void
  deleteProject(workspace: string, project: string): void
  setSecret(model: string, value: string): void
  defineAgent(definition: AgentDefinitionInput): void
  deleteAgentDefinition(id: string): void
  createAgentInstance(agent: string, workspace: string, project: string): void
  deleteAgentInstance(agent: string, workspace: string, project: string): void
  submitTask(input: TaskInput): string
  clearNotice(): void
}

let wired = false

export const useOfficeStore = create<OfficeStore>((set) => {
  const handleEvent = (event: OfficeEvent): void => {
    set((state) => reduceOfficeEvent(state, event))
  }

  const wire = (): void => {
    if (wired) {
      return
    }
    wired = true
    client.onEvent(handleEvent)
    client.onStateChange((connection) => set({ connection }))
    client.onOpen(() => {
      client.send(officeCommands.subscribeAll())
      client.send(officeCommands.listWorkspaces())
      client.send(officeCommands.listProjects())
      client.send(officeCommands.listModels())
      client.send(officeCommands.listTools())
      client.send(officeCommands.listSecrets())
      client.send(officeCommands.listAgentDefinitions())
      client.send(officeCommands.listAgentInstances())
    })
  }

  return {
    ...EMPTY_OFFICE_DOMAIN,
    connection: "closed",
    connect() {
      wire()
      client.connect()
    },
    createWorkspace(name) {
      client.send(officeCommands.createWorkspace(name))
    },
    deleteWorkspace(slug) {
      client.send(officeCommands.deleteWorkspace(slug))
    },
    createProject(workspace, project) {
      client.send(officeCommands.createProject(workspace, project))
    },
    deleteProject(workspace, project) {
      client.send(officeCommands.deleteProject(workspace, project))
    },
    setSecret(model, value) {
      client.send(officeCommands.setSecret(model, value))
    },
    defineAgent(definition) {
      client.send(officeCommands.defineAgent(definition))
    },
    deleteAgentDefinition(id) {
      client.send(officeCommands.deleteAgentDefinition(id))
    },
    createAgentInstance(agent, workspace, project) {
      client.send(officeCommands.createAgentInstance(agent, workspace, project))
    },
    deleteAgentInstance(agent, workspace, project) {
      client.send(officeCommands.deleteAgentInstance(agent, workspace, project))
    },
    submitTask(input) {
      return client.send(officeCommands.submitTask(input))
    },
    clearNotice() {
      set({ notice: null })
    },
  }
})
