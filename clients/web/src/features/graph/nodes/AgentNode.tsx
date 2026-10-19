import { memo } from "react"
import type { NodeProps } from "@xyflow/react"
import { UserRound } from "lucide-react"

import { useOfficeStore } from "@/lib/office/store"

import type { AgentNode as AgentNodeType } from "../model"
import { focusClassName } from "../state/focus"
import { useGraphFocus } from "../state/focusContext"
import { NodeShell } from "./NodeShell"

export const AgentNode = memo(function AgentNode({ id, data }: NodeProps<AgentNodeType>) {
  const focus = useGraphFocus()
  const modelName = useOfficeStore(
    (state) => state.models.find((item) => item.id === data.model)?.displayName ?? data.model,
  )
  const meta = modelName !== "" ? modelName : data.role
  return (
    <NodeShell
      icon={UserRound}
      kind="agent"
      title={data.name}
      meta={meta}
      source
      target
      className={focusClassName(focus, id, "node")}
    />
  )
})
