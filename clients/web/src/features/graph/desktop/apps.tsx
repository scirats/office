import type { ReactNode } from "react"
import { FolderKanban, Terminal, UserRound, type LucideIcon } from "lucide-react"

import type { WorkspaceDetail } from "../workspace"

export type AppId = "projects" | "agents" | "console"

export interface DesktopApp {
  id: AppId
  title: string
  icon: LucideIcon
  width: number
  height: number
  render: (detail: WorkspaceDetail) => ReactNode
}

function ProjectsApp({ detail }: { detail: WorkspaceDetail }) {
  if (detail.projects.length === 0) {
    return <p className="p-4 text-sm text-muted-foreground">No projects yet.</p>
  }
  return (
    <div className="flex h-full flex-col gap-2 overflow-y-auto p-3">
      {detail.projects.map((project) => (
        <div
          key={project.id}
          className="flex items-center gap-3 rounded-lg border border-border/70 bg-background/60 p-3"
        >
          <span className="grid size-8 shrink-0 place-items-center rounded-md bg-muted text-muted-foreground">
            <FolderKanban className="size-4" strokeWidth={1.75} aria-hidden="true" />
          </span>
          <div className="flex min-w-0 flex-col">
            <p className="truncate text-sm font-medium">{project.name}</p>
            <p className="text-xs text-muted-foreground">
              {project.agents.length} {project.agents.length === 1 ? "agent" : "agents"}
            </p>
          </div>
        </div>
      ))}
    </div>
  )
}

function AgentsApp({ detail }: { detail: WorkspaceDetail }) {
  const agents = detail.projects.flatMap((project) =>
    project.agents.map((agent) => ({ ...agent, project: project.name })),
  )
  if (agents.length === 0) {
    return <p className="p-4 text-sm text-muted-foreground">No agents assigned.</p>
  }
  return (
    <div className="flex h-full flex-col gap-2 overflow-y-auto p-3">
      {agents.map((agent) => (
        <div
          key={`${agent.project}:${agent.id}`}
          className="flex items-center gap-3 rounded-lg border border-border/70 bg-background/60 p-3"
        >
          <span className="grid size-8 shrink-0 place-items-center rounded-md bg-muted text-muted-foreground">
            <UserRound className="size-4" strokeWidth={1.75} aria-hidden="true" />
          </span>
          <div className="flex min-w-0 flex-col">
            <p className="truncate text-sm font-medium">{agent.name}</p>
            <p className="truncate font-mono text-xs text-muted-foreground">
              {agent.role} · {agent.project}
            </p>
          </div>
        </div>
      ))}
    </div>
  )
}

function ConsoleApp({ detail }: { detail: WorkspaceDetail }) {
  const lines = [
    `workspace  ${detail.name}`,
    `user       ${detail.user}`,
    `home       ${detail.home}`,
    `projects   ${detail.projects.length}`,
  ]
  return (
    <div className="flex h-full flex-col gap-1 overflow-y-auto bg-black/40 p-3 font-mono text-xs text-emerald-300/90">
      {lines.map((line) => (
        <p key={line}>{line}</p>
      ))}
    </div>
  )
}

export const DESKTOP_APPS: DesktopApp[] = [
  {
    id: "projects",
    title: "Projects",
    icon: FolderKanban,
    width: 460,
    height: 340,
    render: (detail) => <ProjectsApp detail={detail} />,
  },
  {
    id: "agents",
    title: "Agents",
    icon: UserRound,
    width: 420,
    height: 320,
    render: (detail) => <AgentsApp detail={detail} />,
  },
  {
    id: "console",
    title: "Console",
    icon: Terminal,
    width: 480,
    height: 260,
    render: (detail) => <ConsoleApp detail={detail} />,
  },
]
