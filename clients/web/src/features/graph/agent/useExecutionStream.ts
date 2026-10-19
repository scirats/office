import { useCallback, useEffect, useRef, useState } from "react"

import { subscribeOfficeEvents, useOfficeStore } from "@/lib/office/store"
import { slugify } from "@/lib/utils"

import type { TranscriptKind, TranscriptLine } from "./Transcript"

const MAX_LINES = 500

export type ExecutionStatus = "idle" | "running" | "succeeded" | "failed"

interface ExecutionStream {
  lines: TranscriptLine[]
  status: ExecutionStatus
  run(input: string): void
}

function statusOf(state: string): ExecutionStatus {
  switch (state) {
    case "SUCCEEDED":
      return "succeeded"
    case "FAILED":
    case "CANCELLED":
    case "ORPHANED":
      return "failed"
    default:
      return "running"
  }
}

export function useExecutionStream(agent: string): ExecutionStream {
  const submitTask = useOfficeStore((state) => state.submitTask)
  const [lines, setLines] = useState<TranscriptLine[]>([])
  const [status, setStatus] = useState<ExecutionStatus>("idle")
  const command = useRef<string | null>(null)
  const execution = useRef<string | null>(null)
  const nextLineId = useRef(1)

  const push = useCallback((kind: TranscriptKind, text: string) => {
    setLines((current) => {
      const last = current[current.length - 1]
      const mergeable = kind === "text" || kind === "toolResult"
      if (mergeable && last !== undefined && last.kind === kind) {
        const next = current.slice()
        next[next.length - 1] = { ...last, text: last.text + text }
        return next
      }
      const id = nextLineId.current
      nextLineId.current += 1
      const next = [...current, { id, kind, text }]
      return next.length > MAX_LINES ? next.slice(next.length - MAX_LINES) : next
    })
  }, [])

  useEffect(() => {
    return subscribeOfficeEvents((event) => {
      if (event.type === "task.accepted" && event.correlationId === command.current) {
        execution.current = event.executionId
        return
      }
      if (event.type === "command.rejected" && event.correlationId === command.current) {
        push("error", event.message)
        setStatus("failed")
        return
      }
      const executionId = execution.current
      if (executionId === null) {
        return
      }
      if (event.type === "execution.state_changed" && event.executionId === executionId) {
        setStatus(statusOf(event.current))
      } else if (event.type === "execution.log" && event.executionId === executionId) {
        if (event.stream === "stderr") {
          push("error", event.line)
        } else if (event.stream === "tool") {
          push("tool", event.line)
        } else if (event.stream === "tool-result") {
          push("toolResult", event.line)
        } else {
          push("text", event.line)
        }
      }
    })
  }, [push])

  const run = useCallback(
    (input: string) => {
      setLines([])
      setStatus("running")
      execution.current = null
      command.current = submitTask({
        company: "",
        project: "",
        agent: slugify(agent),
        input,
      })
    },
    [agent, submitTask],
  )

  return { lines, status, run }
}
