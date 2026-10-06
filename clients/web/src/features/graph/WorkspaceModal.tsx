import { useEffect, useState } from "react"
import { Layers, Maximize2, Minimize2, XIcon } from "lucide-react"
import { Dialog as DialogPrimitive } from "radix-ui"

import { Button } from "@/components/ui/button"
import {
  Dialog,
  DialogDescription,
  DialogOverlay,
  DialogPortal,
  DialogTitle,
} from "@/components/ui/dialog"
import { Separator } from "@/components/ui/separator"
import { cn } from "@/lib/utils"

import { WorkspaceDesktop } from "./WorkspaceDesktop"
import type { WorkspaceDetail } from "./workspace"

interface WorkspaceModalProps {
  workspace: WorkspaceDetail | null
  onClose: () => void
}

export function WorkspaceModal({ workspace, onClose }: WorkspaceModalProps) {
  const [expanded, setExpanded] = useState(false)
  const [snapshot, setSnapshot] = useState<WorkspaceDetail | null>(workspace)

  useEffect(() => {
    if (workspace !== null) {
      setSnapshot(workspace)
    }
  }, [workspace])

  useEffect(() => {
    setExpanded(false)
  }, [workspace?.id])

  const detail = workspace ?? snapshot

  return (
    <Dialog open={workspace !== null} onOpenChange={(open) => !open && onClose()}>
      <DialogPortal>
        <DialogOverlay />
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
          <DialogPrimitive.Content
            className={cn(
              "relative flex flex-col overflow-hidden bg-popover text-popover-foreground shadow-2xl outline-none",
              expanded
                ? "h-dvh w-screen rounded-none"
                : "h-[min(80vh,44rem)] w-full max-w-5xl rounded-xl ring-1 ring-foreground/10",
            )}
          >
            <header className="flex items-center gap-3 px-4 py-3">
              <span className="grid size-8 shrink-0 place-items-center rounded-lg bg-muted text-muted-foreground">
                <Layers className="size-4" strokeWidth={1.75} aria-hidden="true" />
              </span>
              <div className="flex min-w-0 flex-col">
                <DialogTitle className="truncate">{detail?.name ?? "Workspace"}</DialogTitle>
                <DialogDescription>
                  {detail === null ? "Workspace" : `${detail.user} · ${detail.home}`}
                </DialogDescription>
              </div>
              <div className="ml-auto flex items-center gap-1">
                <Button
                  variant="ghost"
                  size="icon-sm"
                  aria-label={expanded ? "Collapse workspace" : "Expand workspace"}
                  aria-pressed={expanded}
                  onClick={() => setExpanded((value) => !value)}
                >
                  {expanded ? <Minimize2 /> : <Maximize2 />}
                </Button>
                <DialogPrimitive.Close asChild>
                  <Button variant="ghost" size="icon-sm" aria-label="Close workspace">
                    <XIcon />
                  </Button>
                </DialogPrimitive.Close>
              </div>
            </header>
            <Separator />
            <div className="relative min-h-0 flex-1">
              {detail === null ? null : <WorkspaceDesktop detail={detail} expanded={expanded} />}
            </div>
            <Separator />
            <footer className="flex items-center px-4 py-2 text-xs text-muted-foreground">
              Press{" "}
              <kbd className="mx-1 rounded border border-border bg-muted px-1.5 py-0.5 font-mono text-[10px]">
                Esc
              </kbd>{" "}
              to return to the graph
            </footer>
          </DialogPrimitive.Content>
        </div>
      </DialogPortal>
    </Dialog>
  )
}
