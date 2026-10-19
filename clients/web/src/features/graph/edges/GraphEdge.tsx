import { BaseEdge, getBezierPath, type EdgeProps } from "@xyflow/react"

import { assignmentEndpoints, projectKey } from "../model"
import { focusClassName } from "../state/focus"
import { useGraphFocus } from "../state/focusContext"
import { useGraphStore } from "../state/graphStore"

export function GraphEdge({
  id,
  source,
  target,
  sourceX,
  sourceY,
  targetX,
  targetY,
  sourcePosition,
  targetPosition,
}: EdgeProps) {
  const focus = useGraphFocus()
  const unlinkAgent = useGraphStore((state) => state.unlinkAgent)
  const [path, labelX, labelY] = getBezierPath({
    sourceX,
    sourceY,
    sourcePosition,
    targetX,
    targetY,
    targetPosition,
  })
  const assignment = assignmentEndpoints({ source, target })

  return (
    <>
      <BaseEdge id={id} path={path} className={focusClassName(focus, id, "edge")} />
      {assignment === null ? null : (
        <g
          className="graph-edge-delete nodrag nopan"
          transform={`translate(${labelX}, ${labelY})`}
          onClick={(event) => {
            event.stopPropagation()
            unlinkAgent(assignment.agent, projectKey(assignment.workspace, assignment.project))
          }}
          onMouseDown={(event) => event.stopPropagation()}
          aria-label="Unlink agent from project"
        >
          <circle className="graph-edge-delete-bg" r={9} />
          <line className="graph-edge-delete-mark" x1={-3} y1={-3} x2={3} y2={3} />
          <line className="graph-edge-delete-mark" x1={-3} y1={3} x2={3} y2={-3} />
        </g>
      )}
    </>
  )
}
