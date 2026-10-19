import { useEffect } from "react"
import { Controller, useForm } from "react-hook-form"
import { zodResolver } from "@hookform/resolvers/zod"
import { z } from "zod"

import { Input } from "@/components/ui/input"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import { Textarea } from "@/components/ui/textarea"
import { useOfficeStore } from "@/lib/office/store"

import { ApiKeyField } from "./ApiKeyField"
import type { AgentDefinition } from "@/lib/office/protocol"

const definitionSchema = z.object({
  role: z.string().trim().min(1, "Role is required"),
  model: z.string().min(1, "Model is required"),
  prompt: z.string(),
})

export type DefinitionValues = z.infer<typeof definitionSchema>

export function DefinitionForm({
  definition,
  onSetApiKey,
  onChange,
}: {
  definition: AgentDefinition
  onSetApiKey?(model: string, value: string): void
  onChange(values: DefinitionValues): void
}) {
  const models = useOfficeStore((state) => state.models)
  const {
    register,
    control,
    watch,
    getValues,
    formState: { errors },
  } = useForm<DefinitionValues>({
    defaultValues: {
      role: definition.role,
      model: definition.model,
      prompt: definition.prompt,
    },
    resolver: zodResolver(definitionSchema),
    mode: "onChange",
  })

  useEffect(() => {
    const subscription = watch(() => onChange(getValues()))
    return () => subscription.unsubscribe()
  }, [watch, getValues, onChange])

  const model = watch("model")

  return (
    <div className="flex flex-col gap-4">
      <div className="flex flex-col gap-1.5">
        <label
          htmlFor="agent-role"
          className="text-xs font-medium tracking-wide text-muted-foreground"
        >
          Role
        </label>
        <Input
          id="agent-role"
          aria-invalid={errors.role ? true : undefined}
          {...register("role")}
        />
        {errors.role === undefined ? null : (
          <span className="text-xs text-destructive">{errors.role.message}</span>
        )}
      </div>
      <div className="flex flex-col gap-1.5">
        <label className="text-xs font-medium tracking-wide text-muted-foreground">Model</label>
        <Controller
          control={control}
          name="model"
          render={({ field }) => (
            <Select value={field.value} onValueChange={field.onChange}>
              <SelectTrigger className="w-full" aria-label="Model">
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
        {errors.model === undefined ? null : (
          <span className="text-xs text-destructive">{errors.model.message}</span>
        )}
      </div>
      <ApiKeyField model={model} onSetApiKey={onSetApiKey} />
      <div className="flex flex-col gap-1.5">
        <label
          htmlFor="agent-prompt"
          className="text-xs font-medium tracking-wide text-muted-foreground"
        >
          Prompt
        </label>
        <Textarea
          id="agent-prompt"
          rows={8}
          className="font-mono text-xs"
          {...register("prompt")}
        />
      </div>
    </div>
  )
}
