import { useMemo, type KeyboardEvent } from "react"
import { Controller, useForm } from "react-hook-form"
import { zodResolver } from "@hookform/resolvers/zod"
import { z } from "zod"

import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import { useOfficeStore } from "@/lib/office/store"

import type { DraftKind, DraftSubmission } from "../model"
import { useGraphStore } from "../state/graphStore"

const base = {
  name: z.string(),
  role: z.string(),
  model: z.string(),
  workspace: z.string(),
  project: z.string(),
  agent: z.string(),
}

const schemas = {
  workspace: z.object({ ...base, name: z.string().trim().min(1, "Name is required") }),
  agent: z.object({
    ...base,
    role: z.string().trim().min(1, "Role is required"),
    model: z.string().min(1, "Model is required"),
  }),
  project: z.object({
    ...base,
    workspace: z.string().min(1, "Select a workspace"),
    project: z.string().trim().min(1, "Project is required"),
  }),
} satisfies Record<DraftKind, z.ZodType>

type DraftFormValues = z.infer<typeof schemas.agent>

function FieldError({ message }: { message?: string }) {
  if (message === undefined) {
    return null
  }
  return <span className="text-xs text-destructive">{message}</span>
}

export function DraftForm({ kind }: { kind: DraftKind }) {
  const models = useOfficeStore((state) => state.models)
  const workspaces = useOfficeStore((state) => state.workspaces)
  const commitDraft = useGraphStore((state) => state.commitDraft)
  const cancelDraft = useGraphStore((state) => state.cancelDraft)

  const resolver = useMemo(() => zodResolver(schemas[kind]), [kind])
  const {
    register,
    control,
    handleSubmit,
    formState: { errors, isValid },
  } = useForm<DraftFormValues>({
    defaultValues: {
      name: "",
      role: "",
      model: models[0]?.id ?? "",
      workspace: workspaces[0]?.slug ?? "",
      project: "",
      agent: "",
    },
    resolver,
    mode: "onChange",
  })

  const submit = handleSubmit((values) => {
    const submission: DraftSubmission =
      kind === "workspace"
        ? { kind: "workspace", name: values.name.trim() }
        : kind === "agent"
          ? { kind: "agent", role: values.role.trim(), model: values.model }
          : {
              kind: "project",
              workspace: values.workspace,
              project: values.project.trim(),
            }
    commitDraft(submission)
  })

  const onKeyDown = (event: KeyboardEvent) => {
    if (event.key === "Escape") {
      cancelDraft()
    }
  }

  return (
    <form onSubmit={submit} onKeyDown={onKeyDown} className="flex flex-col gap-2 px-3 py-2.5">
      {kind === "workspace" ? (
        <>
          <Input
            autoFocus
            placeholder="Name…"
            aria-label="Name"
            className="nodrag"
            aria-invalid={errors.name ? true : undefined}
            {...register("name")}
          />
          <FieldError message={errors.name?.message} />
        </>
      ) : null}

      {kind === "agent" ? (
        <>
          <Input
            autoFocus
            placeholder="Role…"
            aria-label="Role"
            className="nodrag"
            aria-invalid={errors.role ? true : undefined}
            {...register("role")}
          />
          <FieldError message={errors.role?.message} />
          <Controller
            control={control}
            name="model"
            render={({ field }) => (
              <Select value={field.value} onValueChange={field.onChange}>
                <SelectTrigger size="sm" className="nodrag w-full" aria-label="Model">
                  <SelectValue placeholder="Select a model" />
                </SelectTrigger>
                <SelectContent>
                  {models.map((item) => (
                    <SelectItem key={item.id} value={item.id}>
                      {item.displayName} · {item.provider}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            )}
          />
          <FieldError message={errors.model?.message} />
        </>
      ) : null}

      {kind === "project" ? (
        <>
          <Controller
            control={control}
            name="workspace"
            render={({ field }) => (
              <Select value={field.value} onValueChange={field.onChange}>
                <SelectTrigger size="sm" className="nodrag w-full" aria-label="Workspace">
                  <SelectValue placeholder="Select a workspace" />
                </SelectTrigger>
                <SelectContent>
                  {workspaces.map((item) => (
                    <SelectItem key={item.slug} value={item.slug}>
                      {item.name}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            )}
          />
          <FieldError message={errors.workspace?.message} />
          <Input
            autoFocus
            placeholder="Project…"
            aria-label="Project"
            className="nodrag"
            aria-invalid={errors.project ? true : undefined}
            {...register("project")}
          />
          <FieldError message={errors.project?.message} />
          {workspaces.length === 0 ? (
            <p className="text-xs text-muted-foreground">
              Create a workspace first. Agents can be linked later.
            </p>
          ) : null}
        </>
      ) : null}

      <div className="flex items-center justify-end gap-2">
        <Button type="button" variant="ghost" size="sm" className="nodrag" onClick={cancelDraft}>
          Cancel
        </Button>
        <Button type="submit" size="sm" className="nodrag" disabled={!isValid}>
          Create
        </Button>
      </div>
    </form>
  )
}
