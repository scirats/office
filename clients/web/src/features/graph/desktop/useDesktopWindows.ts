import { useCallback, useEffect, useRef, useState, type RefObject } from "react"
import WinBox from "winbox"

import type { AppId, DesktopApp } from "./apps"

export type WinBoxInstance = InstanceType<typeof WinBox>

export interface DesktopWindow {
  id: AppId
  container: HTMLDivElement
}

const CASCADE_STEP = 26
const MARGIN = 40
const MIN_WIDTH = 260
const MIN_HEIGHT = 180

function viewportWidth(): number {
  return document.documentElement.clientWidth
}

function viewportHeight(): number {
  return document.documentElement.clientHeight
}

function removeFlag(set: ReadonlySet<AppId>, id: AppId): ReadonlySet<AppId> {
  if (!set.has(id)) {
    return set
  }
  const next = new Set(set)
  next.delete(id)
  return next
}

export function useDesktopWindows(desktopRef: RefObject<HTMLDivElement | null>) {
  const handles = useRef(new Map<AppId, WinBoxInstance>())
  const [windows, setWindows] = useState<DesktopWindow[]>([])
  const [focused, setFocused] = useState<AppId | null>(null)
  const [minimized, setMinimized] = useState<ReadonlySet<AppId>>(new Set())

  const open = useCallback(
    (app: DesktopApp) => {
      const existing = handles.current.get(app.id)
      if (existing !== undefined) {
        if (existing.min) {
          existing.restore()
        } else {
          existing.focus()
        }
        setMinimized((prev) => removeFlag(prev, app.id))
        setFocused(app.id)
        return
      }

      const desktop = desktopRef.current
      if (desktop === null) {
        return
      }
      const container = document.createElement("div")
      container.className = "office-app"
      const offset = handles.current.size * CASCADE_STEP
      const win = new WinBox({
        title: app.title,
        root: desktop,
        mount: container,
        x: MARGIN + offset,
        y: MARGIN + offset,
        width: Math.min(app.width, desktop.clientWidth - MARGIN * 2),
        height: Math.min(app.height, desktop.clientHeight - MARGIN * 2),
        minwidth: MIN_WIDTH,
        minheight: MIN_HEIGHT,
        top: 0,
        left: 0,
        right: Math.round(viewportWidth() - desktop.clientWidth),
        bottom: Math.round(viewportHeight() - desktop.clientHeight),
        class: "office-winbox",
        onfocus: () => setFocused(app.id),
        onminimize: () => setMinimized((prev) => new Set(prev).add(app.id)),
        onrestore: () => setMinimized((prev) => removeFlag(prev, app.id)),
        onclose: () => {
          handles.current.delete(app.id)
          setWindows((prev) => prev.filter((item) => item.id !== app.id))
          setMinimized((prev) => removeFlag(prev, app.id))
          setFocused((prev) => (prev === app.id ? null : prev))
        },
      })
      handles.current.set(app.id, win)
      setWindows((prev) => [...prev, { id: app.id, container }])
      setFocused(app.id)
    },
    [desktopRef],
  )

  const syncLimits = useCallback(() => {
    const desktop = desktopRef.current
    if (desktop === null) {
      return
    }
    const desktopW = desktop.clientWidth
    const desktopH = desktop.clientHeight
    const right = Math.round(viewportWidth() - desktopW)
    const bottom = Math.round(viewportHeight() - desktopH)
    const minimizedWindows: WinBoxInstance[] = []
    for (const win of handles.current.values()) {
      win.top = 0
      win.left = 0
      win.right = right
      win.bottom = bottom
      if (win.min) {
        minimizedWindows.push(win)
        continue
      }
      if (win.max) {
        win.maximize(false)
        win.maximize(true)
        continue
      }
      win.width = Math.min(win.width, desktopW)
      win.height = Math.min(win.height, desktopH)
      win.x = Math.max(0, Math.min(win.x, desktopW - win.width))
      win.y = Math.max(0, Math.min(win.y, desktopH - win.height))
      win.resize()
      win.move()
    }
    if (minimizedWindows.length > 0) {
      const width = Math.min(desktopW / minimizedWindows.length, 250)
      minimizedWindows.forEach((win, index) => {
        win.resize((width + 1) | 0, win.header, true)
        win.move((index * width) | 0, desktopH - win.header, true)
      })
    }
  }, [desktopRef])

  useEffect(() => {
    const desktop = desktopRef.current
    if (desktop === null) {
      return
    }
    const observer = new ResizeObserver(syncLimits)
    observer.observe(desktop)
    window.addEventListener("resize", syncLimits)
    return () => {
      observer.disconnect()
      window.removeEventListener("resize", syncLimits)
    }
  }, [desktopRef, syncLimits])

  useEffect(() => {
    const openHandles = handles.current
    return () => {
      for (const win of openHandles.values()) {
        win.close(true)
      }
      openHandles.clear()
    }
  }, [])

  return { windows, focused, minimized, open }
}
