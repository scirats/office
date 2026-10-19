import { useEffect, useRef } from "react"
import { createPortal } from "react-dom"

import { cn } from "@/lib/utils"

import type { WorkspaceDetail } from "../workspace"
import { DESKTOP_APPS } from "./apps"
import { useDesktopWindows } from "./useDesktopWindows"

import "winbox/dist/css/winbox.min.css"
import "../workspace.css"

export function WorkspaceDesktop({ detail }: { detail: WorkspaceDetail }) {
  const desktopRef = useRef<HTMLDivElement>(null)
  const { windows, focused, minimized, open } = useDesktopWindows(desktopRef)

  useEffect(() => {
    open(DESKTOP_APPS[0])
    open(DESKTOP_APPS[1])
  }, [open])

  return (
    <div
      ref={desktopRef}
      className="office-desktop relative h-full w-full overflow-hidden bg-[radial-gradient(circle_at_20%_20%,oklch(0.3_0.06_264),oklch(0.16_0.02_264)_60%)]"
    >
      <div className="pointer-events-none absolute inset-0 opacity-[0.06] [background-image:linear-gradient(to_right,white_1px,transparent_1px),linear-gradient(to_bottom,white_1px,transparent_1px)] [background-size:32px_32px]" />

      {windows.map(({ id, container }) => {
        const app = DESKTOP_APPS.find((item) => item.id === id)
        return app === undefined ? null : createPortal(app.render(detail), container, id)
      })}

      <div className="absolute bottom-3 left-1/2 z-50 flex -translate-x-1/2 items-center gap-1.5 rounded-2xl border border-white/10 bg-card/70 p-1.5 shadow-lg backdrop-blur-md">
        {DESKTOP_APPS.map((app) => {
          const Icon = app.icon
          const isMinimized = minimized.has(app.id)
          return (
            <button
              key={app.id}
              type="button"
              onClick={() => open(app)}
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
