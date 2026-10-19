import { Layers } from "lucide-react"

import { EntityDialog } from "./EntityDialog"
import { WorkspaceDesktop } from "./desktop/WorkspaceDesktop"
import type { WorkspaceDetail } from "./workspace"

export function WorkspaceModal({
  workspace,
  onClose,
}: {
  workspace: WorkspaceDetail | null
  onClose(): void
}) {
  return (
    <EntityDialog
      entity={workspace}
      icon={Layers}
      size="wide"
      title={(value) => value.name}
      subtitle={(value) => `${value.user} · ${value.home}`}
      onClose={onClose}
    >
      {(detail) => <WorkspaceDesktop detail={detail} />}
    </EntityDialog>
  )
}
