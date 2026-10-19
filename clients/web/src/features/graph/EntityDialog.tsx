import { useState, type ReactNode } from "react"
import { Maximize2, Minimize2, XIcon, type LucideIcon } from "lucide-react"

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

export type EntityDialogSize = "narrow" | "compact" | "wide"

function useLastNonNull<T>(value: T | null): T | null {
  const [last, setLast] = useState<T | null>(value)
  if (value !== null && value !== last) {
    setLast(value)
  }
  return value ?? last
}

interface EntityDialogProps<T extends { id: string }> {
  entity: T | null
  icon: LucideIcon
  title: (entity: T) => string
  subtitle: (entity: T) => ReactNode
  size?: EntityDialogSize
  expandable?: boolean
  onClose(): void
  children: (entity: T, expanded: boolean) => ReactNode
}

export function EntityDialog<T extends { id: string }>({
  entity,
  icon: Icon,
  title,
  subtitle,
  size = "compact",
  expandable = true,
  onClose,
  children,
}: EntityDialogProps<T>) {
  const current = useLastNonNull(entity)
  const [expanded, setExpanded] = useState(false)
  const [trackedId, setTrackedId] = useState<string | null>(null)

  const currentId = current === null ? null : current.id
  if (currentId !== trackedId) {
    setTrackedId(currentId)
    setExpanded(false)
  }

  return (
    <Dialog open={entity !== null} onOpenChange={(open) => !open && onClose()}>
      <DialogContent
        showCloseButton={false}
        className={cn(
          "flex flex-col gap-0 overflow-hidden p-0",
          expanded
            ? "top-0 left-0 right-0 bottom-0 h-dvh w-screen max-w-none translate-x-0 translate-y-0 rounded-none sm:max-w-none"
            : size === "wide"
              ? "h-[min(80vh,44rem)] sm:max-w-7xl"
              : size === "narrow"
                ? "h-[min(80vh,40rem)] sm:max-w-3xl"
                : "h-[min(80vh,40rem)] sm:max-w-6xl",
        )}
      >
        <header className="flex items-center gap-3 px-4 py-3">
          <span className="grid size-8 shrink-0 place-items-center rounded-lg bg-muted text-muted-foreground">
            <Icon className="size-4" strokeWidth={1.75} aria-hidden="true" />
          </span>
          <div className="flex min-w-0 flex-col">
            <DialogTitle className="truncate">{current === null ? "" : title(current)}</DialogTitle>
            <DialogDescription className="truncate">
              {current === null ? "" : subtitle(current)}
            </DialogDescription>
          </div>
          <div className="ml-auto flex items-center gap-1">
            {expandable ? (
              <Button
                variant="ghost"
                size="icon-sm"
                aria-label={expanded ? "Collapse" : "Expand"}
                aria-pressed={expanded}
                onClick={() => setExpanded((value) => !value)}
              >
                {expanded ? <Minimize2 /> : <Maximize2 />}
              </Button>
            ) : null}
            <DialogClose asChild>
              <Button variant="ghost" size="icon-sm" aria-label="Close">
                <XIcon />
              </Button>
            </DialogClose>
          </div>
        </header>
        <Separator />
        <div className="relative min-h-0 flex-1">
          {current === null ? null : children(current, expanded)}
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
