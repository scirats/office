import { useRef, useState } from "react"

import { Input } from "@/components/ui/input"
import { useOfficeStore } from "@/lib/office/store"

export function ApiKeyField({
  model,
  onSetApiKey,
}: {
  model: string
  onSetApiKey?(model: string, value: string): void
}) {
  const configured = useOfficeStore(
    (state) => state.models.find((item) => item.id === model)?.credentialConfigured ?? false,
  )
  const [value, setValue] = useState("")
  const saved = useRef<{ model: string; value: string }>({ model: "", value: "" })

  const save = () => {
    const trimmed = value.trim()
    if (trimmed === "" || onSetApiKey === undefined) {
      return
    }
    if (saved.current.model === model && saved.current.value === trimmed) {
      return
    }
    onSetApiKey(model, trimmed)
    saved.current = { model, value: trimmed }
  }

  return (
    <div className="flex flex-col gap-1.5">
      <label className="text-xs font-medium tracking-wide text-muted-foreground">API key</label>
      <Input
        type="password"
        value={value}
        placeholder={configured ? "••••••••" : "Paste API key"}
        className="font-mono text-xs"
        onChange={(event) => setValue(event.target.value)}
        onBlur={save}
        onKeyDown={(event) => {
          if (event.key === "Enter") {
            save()
          }
        }}
      />
    </div>
  )
}
