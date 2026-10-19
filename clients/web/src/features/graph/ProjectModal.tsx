import { FolderKanban } from "lucide-react"

import { EntityDialog } from "./EntityDialog"
import { ProjectBoard } from "./ProjectBoard"
import type { ProjectDetail } from "./project"
import { buildSampleProjectBoard } from "./projectBoard"

export function ProjectModal({
  project,
  onClose,
}: {
  project: ProjectDetail | null
  onClose(): void
}) {
  return (
    <EntityDialog
      entity={project}
      icon={FolderKanban}
      title={(value) => value.name}
      subtitle={() => "Board"}
      onClose={onClose}
    >
      {(value) => (
        <div className="h-full overflow-x-auto p-4">
          <ProjectBoard columns={import.meta.env.DEV ? buildSampleProjectBoard(value) : []} />
        </div>
      )}
    </EntityDialog>
  )
}
