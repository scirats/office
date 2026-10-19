import { memo } from "react"
import type { NodeProps } from "@xyflow/react"
import { Plus } from "lucide-react"

import { DRAFT_LABELS, NODE_WIDTH, type DraftNode as DraftNodeType } from "../model"
import { DraftForm } from "./DraftForm"

export const DraftNode = memo(function DraftNode({ data }: NodeProps<DraftNodeType>) {
  return (
    <div
      className="rounded-xl border border-primary/50 bg-card shadow-sm"
      style={{ width: NODE_WIDTH }}
    >
      <div className="flex cursor-grab items-center gap-2 border-b border-border/60 px-3 py-2 active:cursor-grabbing">
        <span className="grid size-6 shrink-0 place-items-center rounded-md bg-primary/12 text-primary">
          <Plus className="size-3.5" strokeWidth={2} aria-hidden="true" />
        </span>
        <span className="text-[11px] font-semibold tracking-wide text-muted-foreground">
          {DRAFT_LABELS[data.kind]}
        </span>
      </div>
      <DraftForm kind={data.kind} />
    </div>
  )
})
