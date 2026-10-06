import { Handle, Position } from "@xyflow/react"
import type { LucideIcon } from "lucide-react"

import { NODE_WIDTH } from "../model"

interface NodeShellProps {
  icon: LucideIcon
  kind: string
  title: string
  meta: string
  source?: boolean
  target?: boolean
}

export function NodeShell({
  icon: Icon,
  kind,
  title,
  meta,
  source = false,
  target = false,
}: NodeShellProps) {
  return (
    <div className="rounded-xl border border-border/80 bg-card shadow-xs" style={{ width: NODE_WIDTH }}>
      <div className="flex items-center gap-2 border-b border-border/60 px-3 py-2">
        <span className="grid size-6 shrink-0 place-items-center rounded-md bg-muted text-muted-foreground">
          <Icon className="size-3.5" strokeWidth={1.75} aria-hidden="true" />
        </span>
        <span className="text-[11px] font-semibold tracking-wide text-muted-foreground">
          {kind}
        </span>
      </div>
      <div className="flex flex-col gap-0.5 px-3 py-2.5">
        <p className="truncate text-sm font-medium">{title}</p>
        <p className="truncate font-mono text-xs text-muted-foreground">{meta}</p>
      </div>
      {target ? <Handle type="target" position={Position.Left} className="graph-handle" /> : null}
      {source ? <Handle type="source" position={Position.Right} className="graph-handle" /> : null}
    </div>
  )
}
