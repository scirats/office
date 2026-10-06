export interface WorkspaceAgent {
  id: string
  name: string
  role: string
}

export interface WorkspaceProject {
  id: string
  name: string
  agents: WorkspaceAgent[]
}

export interface WorkspaceDetail {
  id: string
  name: string
  user: string
  home: string
  projects: WorkspaceProject[]
}
