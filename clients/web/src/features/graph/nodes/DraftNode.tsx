import { memo, useState } from "react"
import type { NodeProps } from "@xyflow/react"
import { Plus } from "lucide-react"

import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import {
  DRAFT_LABELS,
  DRAFT_OPTIONS,
  NODE_WIDTH,
  type DraftNode as DraftNodeType,
} from "../model"
import { useGraphActions } from "../state/actions"

export const DraftNode = memo(function DraftNode({ data }: NodeProps<DraftNodeType>) {
  const actions = useGraphActions()
  const options = DRAFT_OPTIONS[data.kind]
  const [name, setName] = useState("")
  const [detail, setDetail] = useState(options[0] ?? "")

  const canCommit = name.trim() !== ""

  const commit = () => {
    if (!canCommit) {
      return
    }
    actions.commitDraft({ name: name.trim(), detail })
  }

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
      <div className="flex flex-col gap-2 px-3 py-2.5">
        <Input
          autoFocus
          value={name}
          placeholder="Title…"
          aria-label="Title"
          className="nodrag"
          onChange={(event) => setName(event.target.value)}
          onKeyDown={(event) => {
            if (event.key === "Enter") {
              commit()
            } else if (event.key === "Escape") {
              actions.cancelDraft()
            }
          }}
        />
        {options.length > 0 ? (
          <Select value={detail} onValueChange={setDetail}>
            <SelectTrigger size="sm" className="nodrag w-full" aria-label="Kind">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {options.map((option) => (
                <SelectItem key={option} value={option}>
                  {option}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        ) : null}
        <div className="flex items-center justify-end gap-2">
          <Button variant="ghost" size="sm" className="nodrag" onClick={actions.cancelDraft}>
            Cancel
          </Button>
          <Button size="sm" className="nodrag" disabled={!canCommit} onClick={commit}>
            Create
          </Button>
        </div>
      </div>
    </div>
  )
})
