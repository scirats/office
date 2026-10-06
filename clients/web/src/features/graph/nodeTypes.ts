import type { NodeTypes } from "@xyflow/react"

import { AgentNode } from "./nodes/AgentNode"
import { DraftNode } from "./nodes/DraftNode"
import { ProjectNode } from "./nodes/ProjectNode"
import { WorkspaceNode } from "./nodes/WorkspaceNode"

export const nodeTypes: NodeTypes = {
  workspace: WorkspaceNode,
  project: ProjectNode,
  agent: AgentNode,
  draft: DraftNode,
}
