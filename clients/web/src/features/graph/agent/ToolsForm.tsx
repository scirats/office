import { useState } from "react"

import { ToolsField } from "./ToolsField"

export function ToolsForm({
  tools,
  onChange,
}: {
  tools: string[]
  onChange(tools: string[]): void
}) {
  const [selected, setSelected] = useState(tools)

  return (
    <div className="flex flex-col gap-1.5">
      <div className="flex items-center justify-between">
        <span className="text-xs font-medium tracking-wide text-muted-foreground">Tools</span>
        <span className="text-xs text-muted-foreground">{selected.length}</span>
      </div>
      <ToolsField
        selected={selected}
        onChange={(next) => {
          setSelected(next)
          onChange(next)
        }}
      />
    </div>
  )
}
