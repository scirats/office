import { useState } from "react"

import type { AgentDefinition } from "@/lib/office/protocol"
import { useOfficeStore } from "@/lib/office/store"
import { cn } from "@/lib/utils"

import { DefinitionForm } from "./DefinitionForm"
import { Playground } from "./Playground"
import { ToolsForm } from "./ToolsForm"
import type { AgentDetail } from "./types"
import { useAgentDefinition } from "./useAgentDefinition"

type ConsoleTab = "definition" | "tools" | "playground"

const TABS: ConsoleTab[] = ["definition", "tools", "playground"]

export function AgentConsoleView({ detail }: { detail: AgentDetail }) {
  const definition = useOfficeStore((state) =>
    state.agentDefinitions.find((item) => item.id === detail.definitionId),
  )

  if (definition === undefined) {
    return (
      <div className="flex h-full items-center justify-center p-4 text-sm text-muted-foreground">
        Agent definition not found.
      </div>
    )
  }

  return <AgentConsole definition={definition} />
}

function AgentConsole({ definition: initial }: { definition: AgentDefinition }) {
  const setSecret = useOfficeStore((state) => state.setSecret)
  const { definition, update } = useAgentDefinition(initial)
  const [tab, setTab] = useState<ConsoleTab>("definition")

  return (
    <div className="flex h-full min-h-0 flex-col">
      <div role="tablist" className="flex gap-1 border-b border-border px-3 py-2">
        {TABS.map((value) => (
          <button
            key={value}
            type="button"
            role="tab"
            aria-selected={tab === value}
            onClick={() => setTab(value)}
            className={cn(
              "rounded-md px-3 py-1 text-xs font-medium capitalize transition-colors",
              tab === value
                ? "bg-muted text-foreground"
                : "text-muted-foreground hover:text-foreground",
            )}
          >
            {value}
          </button>
        ))}
      </div>
      <div className="min-h-0 flex-1 overflow-y-auto p-4">
        {tab === "definition" ? (
          <DefinitionForm definition={definition} onSetApiKey={setSecret} onChange={update} />
        ) : tab === "tools" ? (
          <ToolsForm tools={definition.tools} onChange={(tools) => update({ tools })} />
        ) : (
          <Playground agent={definition.name} />
        )}
      </div>
    </div>
  )
}
