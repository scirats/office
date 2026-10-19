import { DRAFT_KINDS, DRAFT_LABELS, NODE_KIND_LABELS, type DraftKind, type NodeKind } from "./model"

export interface GraphMenuState {
  x: number
  y: number
  target: { kind: NodeKind; id: string } | null
}

const ITEM_CLASS =
  "flex w-full rounded-md px-1.5 py-1 text-left text-sm hover:bg-accent hover:text-accent-foreground"

const DELETE_CLASS =
  "flex w-full rounded-md px-1.5 py-1 text-left text-sm text-destructive hover:bg-destructive/10"

function CreateMenu({ onCreate }: { onCreate(kind: DraftKind): void }) {
  return (
    <>
      <p className="px-1.5 py-1 text-xs font-medium text-muted-foreground">Create</p>
      <div className="-mx-1 my-1 h-px bg-border" />
      {DRAFT_KINDS.map((kind) => (
        <button key={kind} type="button" onClick={() => onCreate(kind)} className={ITEM_CLASS}>
          {DRAFT_LABELS[kind]}
        </button>
      ))}
    </>
  )
}

function NodeMenu({ kind, onDelete }: { kind: NodeKind; onDelete(): void }) {
  return (
    <>
      <p className="px-1.5 py-1 text-xs font-medium text-muted-foreground">
        {NODE_KIND_LABELS[kind]}
      </p>
      <div className="-mx-1 my-1 h-px bg-border" />
      <button type="button" onClick={onDelete} className={DELETE_CLASS}>
        Delete {NODE_KIND_LABELS[kind].toLowerCase()}
      </button>
    </>
  )
}

export function GraphContextMenu({
  menu,
  onCreate,
  onDelete,
  onClose,
}: {
  menu: GraphMenuState
  onCreate(kind: DraftKind): void
  onDelete(kind: NodeKind, id: string): void
  onClose(): void
}) {
  const target = menu.target
  return (
    <>
      <div
        className="fixed inset-0 z-40"
        onClick={onClose}
        onContextMenu={(event) => {
          event.preventDefault()
          onClose()
        }}
      />
      <div
        className="fixed z-50 w-48 rounded-lg bg-popover p-1 text-popover-foreground shadow-md ring-1 ring-foreground/10"
        style={{ left: menu.x, top: menu.y }}
      >
        {target === null ? (
          <CreateMenu onCreate={onCreate} />
        ) : (
          <NodeMenu
            kind={target.kind}
            onDelete={() => {
              onDelete(target.kind, target.id)
              onClose()
            }}
          />
        )}
      </div>
    </>
  )
}
