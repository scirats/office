import { Button } from "@/components/ui/button"
import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog"

import { NODE_KIND_LABELS, type NodeKind } from "./model"

export interface DeleteNodeTarget {
  kind: NodeKind
  id: string
  name: string
}

const DESCRIPTION: Record<NodeKind, string> = {
  workspace:
    "This removes the Linux user and home directory for this workspace and its record. This cannot be undone.",
  project: "This removes the project. It must have no linked agents. This cannot be undone.",
  agent:
    "This removes the agent definition. It must not be linked to any project. This cannot be undone.",
}

export function DeleteNodeDialog({
  target,
  onCancel,
  onConfirm,
}: {
  target: DeleteNodeTarget | null
  onCancel(): void
  onConfirm(target: DeleteNodeTarget): void
}) {
  return (
    <Dialog open={target !== null} onOpenChange={(open) => !open && onCancel()}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>
            {target === null ? "Delete" : `Delete ${NODE_KIND_LABELS[target.kind].toLowerCase()}`}
          </DialogTitle>
          <DialogDescription>
            {target === null ? null : `“${target.name}”. ${DESCRIPTION[target.kind]}`}
          </DialogDescription>
        </DialogHeader>
        <DialogFooter>
          <DialogClose asChild>
            <Button variant="outline" size="sm">
              Cancel
            </Button>
          </DialogClose>
          <Button
            variant="destructive"
            size="sm"
            onClick={() => {
              if (target !== null) {
                onConfirm(target)
              }
            }}
          >
            Delete
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
