import { Bot } from "lucide-react"

import { useOfficeStore } from "@/lib/office/store"

import { EntityDialog } from "../EntityDialog"
import { AgentConsoleView } from "./AgentConsole"
import type { AgentDetail } from "./types"

export function AgentModal({ agent, onClose }: { agent: AgentDetail | null; onClose(): void }) {
  const models = useOfficeStore((state) => state.models)

  return (
    <EntityDialog
      entity={agent}
      icon={Bot}
      size="narrow"
      expandable={false}
      title={(value) => value.name}
      subtitle={(value) => {
        const label = models.find((item) => item.id === value.model)?.displayName ?? value.model
        return label === "" ? "agent" : `agent · ${label}`
      }}
      onClose={onClose}
    >
      {(detail) => <AgentConsoleView key={detail.definitionId} detail={detail} />}
    </EntityDialog>
  )
}
