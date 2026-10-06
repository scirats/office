import { useEffect, useState } from "react"
import { FolderKanban, Maximize2, Minimize2, XIcon } from "lucide-react"

import { Button } from "@/components/ui/button"
import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogTitle,
} from "@/components/ui/dialog"
import { Separator } from "@/components/ui/separator"
import { cn } from "@/lib/utils"

import { ProjectBoard } from "./ProjectBoard"
import { buildProjectBoard, type ProjectDetail } from "./project"

interface ProjectModalProps {
  project: ProjectDetail | null
  onClose: () => void
}

export function ProjectModal({ project, onClose }: ProjectModalProps) {
  const [expanded, setExpanded] = useState(false)
  const [snapshot, setSnapshot] = useState<ProjectDetail | null>(project)

  useEffect(() => {
    if (project !== null) {
      setSnapshot(project)
    }
  }, [project])

  useEffect(() => {
    setExpanded(false)
  }, [project?.id])

  const detail = project ?? snapshot

  return (
    <Dialog open={project !== null} onOpenChange={(open) => !open && onClose()}>
      <DialogContent
        showCloseButton={false}
        className={cn(
          "flex flex-col gap-0 overflow-hidden p-0",
          expanded
            ? "top-0 left-0 right-0 bottom-0 h-dvh w-screen max-w-none translate-x-0 translate-y-0 rounded-none sm:max-w-none"
            : "h-[min(80vh,38rem)] sm:max-w-4xl",
        )}
      >
        <header className="flex items-center gap-3 px-4 py-3">
          <span className="grid size-8 shrink-0 place-items-center rounded-lg bg-muted text-muted-foreground">
            <FolderKanban className="size-4" strokeWidth={1.75} aria-hidden="true" />
          </span>
          <div className="flex min-w-0 flex-col">
            <DialogTitle className="truncate">{detail?.name ?? "Project"}</DialogTitle>
            <DialogDescription>Board</DialogDescription>
          </div>
          <div className="ml-auto flex items-center gap-1">
            <Button
              variant="ghost"
              size="icon-sm"
              aria-label={expanded ? "Collapse project" : "Expand project"}
              aria-pressed={expanded}
              onClick={() => setExpanded((value) => !value)}
            >
              {expanded ? <Minimize2 /> : <Maximize2 />}
            </Button>
            <DialogClose asChild>
              <Button variant="ghost" size="icon-sm" aria-label="Close project">
                <XIcon />
              </Button>
            </DialogClose>
          </div>
        </header>
        <Separator />
        <div className="flex min-h-0 flex-1 flex-col">
          {detail === null ? null : <ProjectBoard columns={buildProjectBoard(detail)} />}
        </div>
        <Separator />
        <footer className="flex items-center px-4 py-2 text-xs text-muted-foreground">
          Press{" "}
          <kbd className="mx-1 rounded border border-border bg-muted px-1.5 py-0.5 font-mono text-[10px]">
            Esc
          </kbd>{" "}
          to return to the graph
        </footer>
      </DialogContent>
    </Dialog>
  )
}
