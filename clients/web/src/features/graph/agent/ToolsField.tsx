import { useState } from "react"
import { Command } from "cmdk"
import { Wrench, XIcon } from "lucide-react"

import { Button } from "@/components/ui/button"
import { useOfficeStore } from "@/lib/office/store"

export function ToolsField({
  selected,
  onChange,
}: {
  selected: string[]
  onChange(selected: string[]): void
}) {
  const catalog = useOfficeStore((state) => state.tools)
  const [query, setQuery] = useState("")
  const [open, setOpen] = useState(false)

  const available = catalog.filter((tool) => !selected.includes(tool.id))
  const byId = new Map(catalog.map((tool) => [tool.id, tool]))

  const add = (id: string) => {
    onChange([...selected, id])
    setQuery("")
    setOpen(false)
  }

  return (
    <div className="flex flex-col gap-2">
      <Command shouldFilter loop className="relative">
        <Command.Input
          value={query}
          onValueChange={setQuery}
          onFocus={() => setOpen(true)}
          onBlur={() => setOpen(false)}
          placeholder="Search tools to add"
          className="flex h-8 w-full rounded-lg border border-transparent bg-muted/50 px-3 text-sm outline-none placeholder:text-muted-foreground focus-visible:bg-background dark:bg-muted/40"
        />
        {open ? (
          <Command.List className="absolute z-20 mt-1 max-h-56 w-full overflow-y-auto rounded-lg border border-border bg-popover p-1 shadow-md">
            <Command.Empty className="px-2 py-1.5 text-xs text-muted-foreground">
              No tools found.
            </Command.Empty>
            {available.map((tool) => (
              <Command.Item
                key={tool.id}
                value={`${tool.name} ${tool.description}`}
                onMouseDown={(event) => event.preventDefault()}
                onSelect={() => add(tool.id)}
                className="flex cursor-pointer items-center gap-2 rounded-md px-2 py-1.5 data-[selected=true]:bg-muted"
              >
                <Wrench className="size-3.5 shrink-0 text-muted-foreground" aria-hidden="true" />
                <span className="flex min-w-0 flex-1 flex-col">
                  <span className="truncate font-mono text-xs font-medium">{tool.name}</span>
                  <span className="truncate text-xs text-muted-foreground">{tool.description}</span>
                </span>
              </Command.Item>
            ))}
          </Command.List>
        ) : null}
      </Command>

      {selected.length === 0 ? (
        <p className="text-xs text-muted-foreground">No tools yet. Search above to add.</p>
      ) : (
        <ul className="flex flex-col">
          {selected.map((id) => (
            <li
              key={id}
              className="group -mx-2 flex items-center gap-2.5 rounded-md px-2 py-1.5 transition-colors hover:bg-muted/40"
            >
              <Wrench
                className="size-3.5 shrink-0 text-muted-foreground"
                strokeWidth={1.75}
                aria-hidden="true"
              />
              <span className="flex min-w-0 flex-1 flex-col">
                <span className="truncate font-mono text-xs font-medium">{id}</span>
                <span className="text-xs leading-snug text-muted-foreground">
                  {byId.get(id)?.description ?? "unknown tool"}
                </span>
              </span>
              <Button
                type="button"
                variant="ghost"
                size="icon-xs"
                className="shrink-0 opacity-0 transition-opacity group-hover:opacity-100 focus-visible:opacity-100"
                aria-label={`Remove ${id}`}
                onClick={() => onChange(selected.filter((item) => item !== id))}
              >
                <XIcon />
              </Button>
            </li>
          ))}
        </ul>
      )}
    </div>
  )
}
