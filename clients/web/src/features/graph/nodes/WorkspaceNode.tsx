import { memo } from "react"
import type { NodeProps } from "@xyflow/react"
import { Layers } from "lucide-react"

import type { WorkspaceNode as WorkspaceNodeType } from "../model"
import { NodeShell } from "./NodeShell"

export const WorkspaceNode = memo(function WorkspaceNode({ data }: NodeProps<WorkspaceNodeType>) {
  const projects = `${data.projectCount} ${data.projectCount === 1 ? "project" : "projects"}`
  const meta = data.home === "" ? projects : data.home
  return <NodeShell icon={Layers} kind="Workspace" title={data.name} meta={meta} source />
})
