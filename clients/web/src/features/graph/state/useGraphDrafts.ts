import { useCallback, useMemo, useState } from "react"
import type { Node, XYPosition } from "@xyflow/react"

import {
  createNode,
  draftNodeId,
  type DraftInput,
  type DraftKind,
  type GraphNode,
} from "../model"

export interface GraphDrafts {
  active: boolean
  kind: DraftKind | null
  position: XYPosition | null
  start: (kind: DraftKind, position: XYPosition) => void
  cancel: () => void
  commit: (input: DraftInput) => GraphNode | null
}

interface DraftPosition {
  kind: DraftKind
  position: XYPosition
}

interface GraphDraftsDependencies {
  addNode: (node: Node) => void
  removeDraftNodes: () => void
  getNodePosition: (id: string) => XYPosition | undefined
  persistPositions: (extra?: { id: string; position: XYPosition }) => void
}

function generateId(name: string): string {
  const slug = name
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "")
  const suffix = crypto.randomUUID().slice(0, 4)
  return slug === "" ? suffix : `${slug}-${suffix}`
}

function draftNode(kind: DraftKind, position: XYPosition): Node {
  return {
    id: draftNodeId(kind),
    type: "draft",
    position,
    data: { kind },
    deletable: false,
    connectable: false,
    selectable: false,
  }
}

export function useGraphDrafts({
  addNode,
  removeDraftNodes,
  getNodePosition,
  persistPositions,
}: GraphDraftsDependencies): GraphDrafts {
  const [draft, setDraft] = useState<DraftPosition | null>(null)

  const cancel = useCallback(() => {
    setDraft(null)
    removeDraftNodes()
  }, [removeDraftNodes])

  const start = useCallback(
    (kind: DraftKind, position: XYPosition) => {
      setDraft({ kind, position })
      removeDraftNodes()
      addNode(draftNode(kind, position))
    },
    [addNode, removeDraftNodes],
  )

  const commit = useCallback(
    (input: DraftInput): GraphNode | null => {
      if (draft === null) {
        return null
      }
      if (draft.kind === "workspace") {
        removeDraftNodes()
        setDraft(null)
        return null
      }
      const position = getNodePosition(draftNodeId(draft.kind)) ?? draft.position
      const node = createNode(draft.kind, generateId(input.name), input, position)
      removeDraftNodes()
      addNode(node)
      persistPositions({ id: node.id, position })
      setDraft(null)
      return node
    },
    [draft, addNode, removeDraftNodes, getNodePosition, persistPositions],
  )

  return useMemo(
    () => ({
      active: draft !== null,
      kind: draft?.kind ?? null,
      position: draft?.position ?? null,
      start,
      cancel,
      commit,
    }),
    [draft, start, cancel, commit],
  )
}
