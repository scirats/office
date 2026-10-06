import { memo } from "react"
import type { NodeProps } from "@xyflow/react"
import { FolderKanban } from "lucide-react"

import type { ProjectNode as ProjectNodeType } from "../model"
import { NodeShell } from "./NodeShell"

export const ProjectNode = memo(function ProjectNode({ data }: NodeProps<ProjectNodeType>) {
  const agents = `${data.agentCount} ${data.agentCount === 1 ? "agent" : "agents"}`
  return <NodeShell icon={FolderKanban} kind="Project" title={data.name} meta={agents} source target />
})
