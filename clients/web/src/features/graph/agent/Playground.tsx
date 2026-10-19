import { useForm } from "react-hook-form"
import { Play } from "lucide-react"

import { Button } from "@/components/ui/button"
import { Textarea } from "@/components/ui/textarea"
import { cn } from "@/lib/utils"

import { Transcript, TranscriptSurface } from "./Transcript"
import { useExecutionStream, type ExecutionStatus } from "./useExecutionStream"

interface PlaygroundForm {
  input: string
}

const STATUS_LABEL: Record<ExecutionStatus, string> = {
  idle: "",
  running: "Running…",
  succeeded: "Done",
  failed: "Failed",
}

export function Playground({ agent }: { agent: string }) {
  const {
    register,
    handleSubmit,
    reset,
    formState: { isValid },
  } = useForm<PlaygroundForm>({ mode: "onChange", defaultValues: { input: "" } })

  const { lines, status, run } = useExecutionStream(agent)

  const submit = handleSubmit((values) => {
    run(values.input)
    reset({ input: "" })
  })

  return (
    <form className="flex h-full min-h-0 flex-col gap-3" onSubmit={submit}>
      <div className="flex flex-col gap-1.5">
        <label
          htmlFor="playground-input"
          className="text-xs font-medium tracking-wide text-muted-foreground"
        >
          Input
        </label>
        <Textarea
          id="playground-input"
          rows={3}
          placeholder={`Task for ${agent}…`}
          {...register("input", { required: true })}
        />
      </div>
      <div className="flex min-h-0 flex-1 flex-col gap-1.5">
        <div className="flex items-center justify-between">
          <span className="text-xs font-medium tracking-wide text-muted-foreground">Output</span>
          <div className="flex items-center gap-3">
            <span
              className={cn(
                "text-xs",
                status === "failed" ? "text-destructive" : "text-muted-foreground",
              )}
            >
              {STATUS_LABEL[status]}
            </span>
            <Button type="submit" size="sm" disabled={!isValid}>
              <Play className="size-3.5" strokeWidth={1.75} aria-hidden="true" />
              Run
            </Button>
          </div>
        </div>
        <TranscriptSurface>
          <Transcript lines={lines} emptyMessage="Run the agent to see the output." />
        </TranscriptSurface>
      </div>
    </form>
  )
}
