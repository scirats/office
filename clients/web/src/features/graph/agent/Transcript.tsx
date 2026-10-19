import { cn } from "@/lib/utils"

import { Markdown } from "./Markdown"

export type TranscriptKind = "text" | "tool" | "toolResult" | "error"

export interface TranscriptLine {
  id: number
  kind: TranscriptKind
  text: string
}

const KIND_CLASS: Record<Exclude<TranscriptKind, "text">, string> = {
  tool: "text-primary",
  toolResult: "pl-3 text-muted-foreground",
  error: "text-destructive",
}

export function Transcript({
  lines,
  emptyMessage,
}: {
  lines: TranscriptLine[]
  emptyMessage: string
}) {
  if (lines.length === 0) {
    return <p className="text-muted-foreground">{emptyMessage}</p>
  }
  return (
    <>
      {lines.map((line) =>
        line.kind === "text" ? (
          <Markdown key={line.id}>{line.text}</Markdown>
        ) : (
          <p key={line.id} className={cn("whitespace-pre-wrap", KIND_CLASS[line.kind])}>
            {line.text}
          </p>
        ),
      )}
    </>
  )
}

export function TranscriptSurface({ children }: { children: React.ReactNode }) {
  return (
    <div className="min-h-0 flex-1 overflow-y-auto rounded-lg border border-border bg-muted/30 p-3 font-mono text-[11px] leading-relaxed">
      {children}
    </div>
  )
}
