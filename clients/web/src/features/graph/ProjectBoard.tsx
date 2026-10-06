import { UserRound } from "lucide-react"

import { cn } from "@/lib/utils"

import type { TaskCard, TaskColumn, TaskPriority } from "./project"

const PRIORITY_DOT: Record<TaskPriority, string> = {
  high: "bg-destructive",
  medium: "bg-primary",
  low: "bg-muted-foreground",
}

const PRIORITY_LABEL: Record<TaskPriority, string> = {
  high: "High priority",
  medium: "Medium priority",
  low: "Low priority",
}

function BoardCard({ card }: { card: TaskCard }) {
  return (
    <article className="flex flex-col gap-2 rounded-lg border border-border/80 bg-card p-2.5 shadow-xs">
      <p className="text-sm leading-snug font-medium">{card.title}</p>
      <div className="flex items-center gap-1.5 text-xs text-muted-foreground">
        <UserRound className="size-3" strokeWidth={1.75} aria-hidden="true" />
        <span className="truncate">{card.agent}</span>
      </div>
      <div className="flex items-center justify-between gap-2">
        <span className="font-mono text-[10px] text-muted-foreground">{card.execution}</span>
        <span
          className={cn("size-2 rounded-full", PRIORITY_DOT[card.priority])}
          title={PRIORITY_LABEL[card.priority]}
        >
          <span className="sr-only">{PRIORITY_LABEL[card.priority]}</span>
        </span>
      </div>
    </article>
  )
}

export function ProjectBoard({ columns }: { columns: TaskColumn[] }) {
  return (
    <div className="flex h-full min-h-0 gap-3 overflow-x-auto p-4">
      {columns.map((column) => (
        <section
          key={column.id}
          className="flex w-72 shrink-0 flex-col gap-2 rounded-lg border border-border/60 bg-muted/30 p-2"
        >
          <header className="flex items-center justify-between px-1">
            <h3 className="text-xs font-medium tracking-wide text-muted-foreground">
              {column.title}
            </h3>
            <span className="text-xs text-muted-foreground">{column.cards.length}</span>
          </header>
          <div className="flex min-h-0 flex-1 flex-col gap-2 overflow-y-auto">
            {column.cards.length === 0 ? (
              <p className="px-1 py-2 text-xs text-muted-foreground">No tasks</p>
            ) : (
              column.cards.map((card) => <BoardCard key={card.id} card={card} />)
            )}
          </div>
        </section>
      ))}
    </div>
  )
}
