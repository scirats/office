import { memo } from "react"
import type { NodeProps } from "@xyflow/react"
import { UserRound } from "lucide-react"

import type { AgentNode as AgentNodeType } from "../model"
import { NodeShell } from "./NodeShell"

export const AgentNode = memo(function AgentNode({ data }: NodeProps<AgentNodeType>) {
  return <NodeShell icon={UserRound} kind="Agent" title={data.name} meta={data.role} target />
})
