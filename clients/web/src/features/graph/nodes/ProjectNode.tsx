import { memo } from "react"
import type { NodeProps } from "@xyflow/react"
import { FolderKanban } from "lucide-react"

import type { ProjectNode as ProjectNodeType } from "../model"
import { focusClassName } from "../state/focus"
import { useGraphFocus } from "../state/focusContext"
import { useGraphStore } from "../state/graphStore"
import { NodeShell } from "./NodeShell"

export const ProjectNode = memo(function ProjectNode({ id, data }: NodeProps<ProjectNodeType>) {
  const focus = useGraphFocus()
  const agents = useGraphStore((state) => state.childCounts[id] ?? 0)
  const meta = `${agents} ${agents === 1 ? "agent" : "agents"}`
  return (
    <NodeShell
      icon={FolderKanban}
      kind="project"
      title={data.name}
      meta={meta}
      source
      target
      className={focusClassName(focus, id, "node")}
    />
  )
})
