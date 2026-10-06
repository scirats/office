import { useEffect, useMemo, useState } from "react"
import { Bot, Maximize2, Minimize2, XIcon } from "lucide-react"

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

import { AgentView } from "./AgentView"
import { buildAgentConsole, type AgentDefinitionView, type AgentDetail } from "./agent"

interface AgentModalProps {
  agent: AgentDetail | null
  onClose: () => void
  onSaveDefinition?: (definition: AgentDefinitionView) => void
}

export function AgentModal({ agent, onClose, onSaveDefinition }: AgentModalProps) {
  const [expanded, setExpanded] = useState(false)
  const [snapshot, setSnapshot] = useState<AgentDetail | null>(agent)

  useEffect(() => {
    if (agent !== null) {
      setSnapshot(agent)
    }
  }, [agent])

  useEffect(() => {
    setExpanded(false)
  }, [agent?.id])

  const detail = agent ?? snapshot
  const model = useMemo(
    () => (detail === null ? null : buildAgentConsole(detail)),
    [detail],
  )

  return (
    <Dialog open={agent !== null} onOpenChange={(open) => !open && onClose()}>
      <DialogContent
        showCloseButton={false}
        className={cn(
          "flex flex-col gap-0 overflow-hidden p-0",
          expanded
            ? "top-0 left-0 right-0 bottom-0 h-dvh w-screen max-w-none translate-x-0 translate-y-0 rounded-none sm:max-w-none"
            : "h-[min(80vh,40rem)] sm:max-w-4xl",
        )}
      >
        <header className="flex items-center gap-3 px-4 py-3">
          <span className="grid size-8 shrink-0 place-items-center rounded-lg bg-muted text-muted-foreground">
            <Bot className="size-4" strokeWidth={1.75} aria-hidden="true" />
          </span>
          <div className="flex min-w-0 flex-col">
            <DialogTitle className="truncate">{detail?.name ?? "Agent"}</DialogTitle>
            <DialogDescription>
              {detail === null ? "Agent" : `agent · ${detail.role}`}
            </DialogDescription>
          </div>
          <div className="ml-auto flex items-center gap-1">
            <Button
              variant="ghost"
              size="icon-sm"
              aria-label={expanded ? "Collapse agent" : "Expand agent"}
              aria-pressed={expanded}
              onClick={() => setExpanded((value) => !value)}
            >
              {expanded ? <Minimize2 /> : <Maximize2 />}
            </Button>
            <DialogClose asChild>
              <Button variant="ghost" size="icon-sm" aria-label="Close agent">
                <XIcon />
              </Button>
            </DialogClose>
          </div>
        </header>
        <Separator />
        <div className="relative min-h-0 flex-1">
          {model === null ? null : (
            <AgentView model={model} onSave={onSaveDefinition} expanded={expanded} />
          )}
        </div>
        <Separator />
        <footer className="flex items-center px-4 py-2 text-xs text-muted-foreground">
          Instances are read-only · press{" "}
          <kbd className="mx-1 rounded border border-border bg-muted px-1.5 py-0.5 font-mono text-[10px]">
            Esc
          </kbd>{" "}
          to return to the graph
        </footer>
      </DialogContent>
    </Dialog>
  )
}
