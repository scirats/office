import { memo } from "react"
import type { NodeProps } from "@xyflow/react"
import { Layers } from "lucide-react"

import type { WorkspaceNode as WorkspaceNodeType } from "../model"
import { focusClassName } from "../state/focus"
import { useGraphFocus } from "../state/focusContext"
import { useGraphStore } from "../state/graphStore"
import { NodeShell } from "./NodeShell"

export const WorkspaceNode = memo(function WorkspaceNode({
  id,
  data,
}: NodeProps<WorkspaceNodeType>) {
  const focus = useGraphFocus()
  const projects = useGraphStore((state) => state.childCounts[id] ?? 0)
  const meta =
    data.home !== "" ? data.home : `${projects} ${projects === 1 ? "project" : "projects"}`
  return (
    <NodeShell
      icon={Layers}
      kind="workspace"
      title={data.name}
      meta={meta}
      source
      className={focusClassName(focus, id, "node")}
    />
  )
})
