import { useCallback, useEffect, useMemo, useRef, useState, type ReactNode } from "react"
import { createRoot, type Root } from "react-dom/client"
import {
  FolderKanban,
  Terminal,
  UserRound,
  type LucideIcon,
} from "lucide-react"
import WinBox from "winbox"

import { cn } from "@/lib/utils"

import type { WorkspaceDetail } from "./workspace"

import "winbox/dist/css/winbox.min.css"
import "./workspace.css"

type WinBoxInstance = ReturnType<typeof WinBox>
type AppId = "projects" | "agents" | "console"

interface DesktopApp {
  id: AppId
  title: string
  icon: LucideIcon
  width: number
  height: number
  render: (detail: WorkspaceDetail) => ReactNode
}

interface WindowHandle {
  win: WinBoxInstance
  root: Root
}

interface DesktopLimits {
  left: number
  top: number
  right: number
  bottom: number
  width: number
  height: number
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
    `${detail.user}@office ~ % office status`,
    `workspace  ${detail.name}`,
    `home       ${detail.home}`,
    `projects   ${detail.projects.length}`,
    "daemon     listening on /run/office/office.sock",
    "",
    `${detail.user}@office ~ % _`,
  ]
  return (
    <div className="flex h-full flex-col gap-1 overflow-y-auto bg-black/40 p-3 font-mono text-xs text-emerald-300/90">
      {lines.map((line) => (
        <p key={line}>{line}</p>
      ))}
    </div>
  )
}

const DESKTOP_APPS: DesktopApp[] = [
  { id: "projects", title: "Projects", icon: FolderKanban, width: 460, height: 340, render: (detail) => <ProjectsApp detail={detail} /> },
  { id: "agents", title: "Agents", icon: UserRound, width: 420, height: 320, render: (detail) => <AgentsApp detail={detail} /> },
  { id: "console", title: "Console", icon: Terminal, width: 480, height: 260, render: (detail) => <ConsoleApp detail={detail} /> },
]

function desktopLimits(desktop: HTMLDivElement | null): DesktopLimits | null {
  if (desktop === null) {
    return null
  }
  const rect = desktop.getBoundingClientRect()
  return {
    left: Math.round(rect.left),
    top: Math.round(rect.top),
    right: Math.round(window.innerWidth - rect.right),
    bottom: Math.round(window.innerHeight - rect.bottom),
    width: rect.width,
    height: rect.height,
  }
}

export function WorkspaceDesktop({ detail, expanded }: { detail: WorkspaceDetail; expanded: boolean }) {
  const desktopRef = useRef<HTMLDivElement>(null)
  const windowsRef = useRef(new Map<AppId, WindowHandle>())
  const [focused, setFocused] = useState<AppId | null>(null)
  const [minimized, setMinimized] = useState<ReadonlySet<AppId>>(new Set())
  const detailRef = useRef(detail)
  detailRef.current = detail

  const openWindow = useCallback((app: DesktopApp) => {
    const existing = windowsRef.current.get(app.id)
    if (existing !== undefined) {
      if (existing.win.min) {
        existing.win.restore()
      } else if (existing.win.hidden) {
        existing.win.show(true)
      }
      existing.win.focus()
      setMinimized((prev) => {
        if (!prev.has(app.id)) {
          return prev
        }
        const next = new Set(prev)
        next.delete(app.id)
        return next
      })
      setFocused(app.id)
      return
    }
    const limits = desktopLimits(desktopRef.current)
    const container = document.createElement("div")
    container.className = "office-app"
    const root = createRoot(container)
    root.render(app.render(detailRef.current))
    const offset = windowsRef.current.size * 26
    const width = Math.min(app.width, (limits?.width ?? app.width) - 48)
    const height = Math.min(app.height, (limits?.height ?? app.height) - 48)
    const win = new WinBox({
      title: app.title,
      root: desktopRef.current ?? document.body,
      mount: container,
      x: (limits?.left ?? 0) + 40 + offset,
      y: (limits?.top ?? 0) + 32 + offset,
      width,
      height,
      minwidth: 260,
      minheight: 180,
      top: limits?.top ?? 0,
      left: limits?.left ?? 0,
      right: limits?.right ?? 0,
      bottom: limits?.bottom ?? 0,
      class: "office-winbox",
      onfocus: () => setFocused(app.id),
      onminimize: () => setMinimized((prev) => new Set(prev).add(app.id)),
      onrestore: () =>
        setMinimized((prev) => {
          if (!prev.has(app.id)) {
            return prev
          }
          const next = new Set(prev)
          next.delete(app.id)
          return next
        }),
      onclose: () => {
        windowsRef.current.delete(app.id)
        queueMicrotask(() => root.unmount())
        setMinimized((prev) => {
          if (!prev.has(app.id)) {
            return prev
          }
          const next = new Set(prev)
          next.delete(app.id)
          return next
        })
        setFocused((prev) => (prev === app.id ? null : prev))
        return false
      },
    })
    windowsRef.current.set(app.id, { win, root })
    setFocused(app.id)
  }, [])

  const syncLimits = useCallback(() => {
    const limits = desktopLimits(desktopRef.current)
    if (limits === null) {
      return
    }
    for (const { win } of windowsRef.current.values()) {
      win.top = limits.top
      win.left = limits.left
      win.right = limits.right
      win.bottom = limits.bottom
      if (win.max) {
        win.maximize(false)
        win.maximize(true)
      } else {
        win.resize()
        win.move()
      }
    }
  }, [])

  useEffect(() => {
    openWindow(DESKTOP_APPS[0])
    openWindow(DESKTOP_APPS[1])
    return () => {
      for (const { win, root } of windowsRef.current.values()) {
        win.close(true)
        queueMicrotask(() => root.unmount())
      }
      windowsRef.current.clear()
    }
  }, [openWindow])

  useEffect(() => {
    syncLimits()
    const desktop = desktopRef.current
    if (desktop === null) {
      return
    }
    const observer = new ResizeObserver(() => syncLimits())
    observer.observe(desktop)
    window.addEventListener("resize", syncLimits)
    return () => {
      observer.disconnect()
      window.removeEventListener("resize", syncLimits)
    }
  }, [expanded, syncLimits])

  const apps = useMemo(() => DESKTOP_APPS, [])

  return (
    <div
      ref={desktopRef}
      className="relative h-full w-full overflow-hidden bg-[radial-gradient(circle_at_20%_20%,oklch(0.3_0.06_264),oklch(0.16_0.02_264)_60%)]"
    >
      <div className="pointer-events-none absolute inset-0 opacity-[0.06] [background-image:linear-gradient(to_right,white_1px,transparent_1px),linear-gradient(to_bottom,white_1px,transparent_1px)] [background-size:32px_32px]" />
      <div className="absolute bottom-3 left-1/2 z-50 flex -translate-x-1/2 items-center gap-1.5 rounded-2xl border border-white/10 bg-card/70 p-1.5 shadow-lg backdrop-blur-md">
        {apps.map((app) => {
          const Icon = app.icon
          const isMinimized = minimized.has(app.id)
          return (
            <button
              key={app.id}
              type="button"
              onClick={() => openWindow(app)}
              title={app.title}
              className={cn(
                "group relative grid size-11 place-items-center rounded-xl border border-white/10 transition-colors",
                focused === app.id ? "bg-muted" : "bg-card/60 hover:bg-muted/70",
              )}
            >
              <Icon className="size-5 text-foreground" strokeWidth={1.75} aria-hidden="true" />
              {isMinimized ? (
                <span className="absolute -bottom-0.5 size-1 rounded-full bg-primary" />
              ) : null}
            </button>
          )
        })}
      </div>
    </div>
  )
}
