import { BaseEdge, EdgeLabelRenderer, getBezierPath, type EdgeProps } from "@xyflow/react"
import { X } from "lucide-react"

import { useGraphActions } from "../state/actions"

export function GraphEdge({
  id,
  sourceX,
  sourceY,
  sourcePosition,
  targetX,
  targetY,
  targetPosition,
  markerEnd,
  interactionWidth,
  selected,
}: EdgeProps) {
  const actions = useGraphActions()
  const [path, labelX, labelY] = getBezierPath({
    sourceX,
    sourceY,
    sourcePosition,
    targetX,
    targetY,
    targetPosition,
  })

  return (
    <>
      <BaseEdge
        id={id}
        path={path}
        markerEnd={markerEnd}
        interactionWidth={interactionWidth}
      />
      {selected ? (
        <EdgeLabelRenderer>
          <button
            type="button"
            className="nodrag nopan absolute grid size-5 cursor-pointer place-items-center rounded-full border border-border bg-muted text-muted-foreground shadow-sm transition-colors hover:bg-destructive hover:text-background"
            style={{
              transform: `translate(-50%, -50%) translate(${labelX}px, ${labelY}px)`,
              pointerEvents: "all",
            }}
            aria-label="Remove link"
            title="Remove link"
            onClick={() => actions.removeEdge(id)}
          >
            <X className="size-3" strokeWidth={2.5} aria-hidden="true" />
          </button>
        </EdgeLabelRenderer>
      ) : null}
    </>
  )
}
