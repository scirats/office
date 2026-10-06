import { useEffect, useMemo, useRef, useState } from "react"
import {
  Background,
  BackgroundVariant,
  Handle,
  Position,
  ReactFlow,
  ReactFlowProvider,
  type Edge,
  type Node,
  type NodeProps,
  type NodeTypes,
} from "@xyflow/react"
import { Bot, FolderKanban, Plus, Search, Wrench, XIcon } from "lucide-react"

import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Textarea } from "@/components/ui/textarea"
import { cn } from "@/lib/utils"

import {
  AGENT_TOOL_CATALOG,
  type AgentConsoleModel,
  type AgentDefinitionView,
  type AgentInstance,
  type AgentSession,
  type AgentTool,
  type TranscriptKind,
} from "./agent"

const AGENT_NODE_ID = "agent"
const INSTANCE_PREFIX = "instance:"

type AgentNodeData = { name: string; role: string }
type InstanceNodeData = {
  project: string
  state: "running" | "idle"
  task: string | null
  queue: number
}

type AgentFlowNode = Node<AgentNodeData, "agentCore">
type InstanceFlowNode = Node<InstanceNodeData, "agentInstance">
type FlowNode = AgentFlowNode | InstanceFlowNode

const CENTER_HANDLE = {
  left: "50%",
  top: "50%",
  right: "auto",
  bottom: "auto",
  transform: "translate(-50%, -50%)",
  opacity: 0,
  width: 1,
  height: 1,
  minWidth: 0,
  minHeight: 0,
  border: 0,
  background: "transparent",
} as const

function AgentCoreNode({ data, selected }: NodeProps<AgentFlowNode>) {
  return (
    <div
      className={cn(
        "flex w-48 flex-col items-center gap-1 rounded-2xl border bg-card px-4 py-3 text-center shadow-md transition-colors",
        selected ? "border-primary" : "border-border/80",
      )}
    >
      <span className="grid size-9 place-items-center rounded-full bg-primary/15 text-primary">
        <Bot className="size-4" strokeWidth={1.75} aria-hidden="true" />
      </span>
      <p className="w-full truncate text-sm font-medium">{data.name}</p>
      <p className="w-full truncate font-mono text-[11px] text-muted-foreground">{data.role}</p>
      <Handle type="source" position={Position.Right} style={CENTER_HANDLE} />
    </div>
  )
}

function InstanceNode({ data, selected }: NodeProps<InstanceFlowNode>) {
  return (
    <div
      className={cn(
        "flex w-44 flex-col gap-1 rounded-xl border bg-card px-3 py-2 shadow-sm transition-colors",
        selected ? "border-primary" : "border-border/80",
      )}
    >
      <span className="flex items-center gap-2">
        <span
          className={cn(
            "size-2 shrink-0 rounded-full",
            data.state === "running" ? "animate-pulse bg-primary" : "bg-muted-foreground/50",
          )}
        />
        <FolderKanban className="size-3.5 shrink-0 text-muted-foreground" strokeWidth={1.75} aria-hidden="true" />
        <span className="truncate text-sm font-medium">{data.project}</span>
        {data.queue > 0 ? (
          <span className="ml-auto shrink-0 rounded-md border border-border/70 px-1.5 text-[10px] text-muted-foreground">
            {data.queue}
          </span>
        ) : null}
      </span>
      <span className="truncate pl-4 font-mono text-[11px] text-muted-foreground">
        {data.task ?? "idle"}
      </span>
      <Handle type="target" position={Position.Left} style={CENTER_HANDLE} />
    </div>
  )
}

const nodeTypes: NodeTypes = {
  agentCore: AgentCoreNode,
  agentInstance: InstanceNode,
}

function buildNodes(model: AgentConsoleModel, definition: AgentDefinitionView): FlowNode[] {
  const count = model.instances.length
  const radius = Math.max(220, count * 48)
  const nodes: FlowNode[] = [
    {
      id: AGENT_NODE_ID,
      type: "agentCore",
      position: { x: -96, y: -40 },
      data: { name: definition.name, role: definition.role },
      draggable: false,
    },
  ]

  model.instances.forEach((instance, index) => {
    let x: number
    let y: number
    if (count === 1) {
      x = radius
      y = 0
    } else {
      const angle = -Math.PI / 2 + (Math.PI * 2 * index) / count
      x = Math.cos(angle) * radius
      y = Math.sin(angle) * radius
    }
    nodes.push({
      id: `${INSTANCE_PREFIX}${instance.id}`,
      type: "agentInstance",
      position: { x: x - 88, y: y - 28 },
      data: {
        project: instance.projectName,
        state: instance.state,
        task: instance.session?.task ?? null,
        queue: instance.queue,
      },
      draggable: false,
    })
  })

  return nodes
}

function buildEdges(model: AgentConsoleModel, selectedId: string | null): Edge[] {
  return model.instances.map((instance) => {
    const target = `${INSTANCE_PREFIX}${instance.id}`
    const active = selectedId === target
    return {
      id: `edge:${instance.id}`,
      source: AGENT_NODE_ID,
      target,
      type: "straight",
      animated: instance.state === "running",
      style: {
        stroke: active ? "var(--primary)" : "var(--border)",
        strokeWidth: active ? 2.5 : 1.5,
      },
    }
  })
}

const TRANSCRIPT_PREFIX: Record<TranscriptKind, string> = {
  cmd: "",
  out: "",
  ok: "✓ ",
  edit: "± ",
  warn: "! ",
  meta: "",
}

const TRANSCRIPT_CLASS: Record<TranscriptKind, string> = {
  cmd: "text-foreground",
  out: "text-muted-foreground",
  ok: "text-primary",
  edit: "text-foreground",
  warn: "text-destructive",
  meta: "text-muted-foreground/70",
}

function SessionTranscript({ session }: { session: AgentSession }) {
  return (
    <div className="flex min-h-0 flex-1 flex-col gap-1 overflow-y-auto bg-muted/30 p-3 font-mono text-[11px] leading-relaxed">
      {session.transcript.map((line, index) => (
        <p
          key={`${index}:${line.text}`}
          className={cn("whitespace-pre-wrap", TRANSCRIPT_CLASS[line.kind])}
        >
          {TRANSCRIPT_PREFIX[line.kind]}
          {line.text}
        </p>
      ))}
    </div>
  )
}

function ToolsEditor({
  tools,
  onChange,
}: {
  tools: AgentTool[]
  onChange: (tools: AgentTool[]) => void
}) {
  const [query, setQuery] = useState("")
  const [open, setOpen] = useState(false)
  const selected = new Set(tools.map((tool) => tool.name))
  const needle = query.trim().toLowerCase()
  const matches = AGENT_TOOL_CATALOG.filter(
    (tool) =>
      !selected.has(tool.name) &&
      (needle === "" ||
        tool.name.toLowerCase().includes(needle) ||
        tool.description.toLowerCase().includes(needle)),
  )

  const add = (tool: AgentTool) => {
    onChange([...tools, tool])
    setQuery("")
    setOpen(false)
  }

  const remove = (name: string) => {
    onChange(tools.filter((tool) => tool.name !== name))
  }

  return (
    <div className="flex flex-col gap-2">
      <div className="relative">
        <Search
          className="pointer-events-none absolute top-1/2 left-2 size-3.5 -translate-y-1/2 text-muted-foreground"
          strokeWidth={1.75}
          aria-hidden="true"
        />
        <Input
          value={query}
          placeholder="Search tools to add"
          className="border-transparent bg-muted/50 pl-7 focus-visible:bg-background dark:bg-muted/40"
          onChange={(event) => {
            setQuery(event.target.value)
            setOpen(true)
          }}
          onFocus={() => setOpen(true)}
          onBlur={() => setOpen(false)}
          onKeyDown={(event) => {
            if (event.key === "Enter" && matches.length > 0) {
              event.preventDefault()
              add(matches[0])
            } else if (event.key === "Escape") {
              setOpen(false)
            }
          }}
        />
        {open && matches.length > 0 ? (
          <ul className="absolute z-20 mt-1 max-h-56 w-full overflow-y-auto rounded-lg border border-border bg-popover p-1 shadow-md">
            {matches.map((tool) => (
              <li key={tool.name}>
                <button
                  type="button"
                  onMouseDown={(event) => event.preventDefault()}
                  onClick={() => add(tool)}
                  className="flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-left transition-colors hover:bg-muted"
                >
                  <Plus className="size-3.5 shrink-0 text-muted-foreground" aria-hidden="true" />
                  <span className="flex min-w-0 flex-1 flex-col">
                    <span className="truncate font-mono text-xs font-medium">{tool.name}</span>
                    <span className="truncate text-xs text-muted-foreground">
                      {tool.description}
                    </span>
                  </span>
                </button>
              </li>
            ))}
          </ul>
        ) : null}
      </div>
      {tools.length === 0 ? (
        <p className="text-xs text-muted-foreground">No tools yet. Search above to add.</p>
      ) : (
        <ul className="flex flex-col">
          {tools.map((tool) => (
            <li
              key={tool.name}
              className="group -mx-2 flex items-center gap-2.5 rounded-md px-2 py-1.5 transition-colors hover:bg-muted/40"
            >
              <Wrench
                className="size-3.5 shrink-0 text-muted-foreground"
                strokeWidth={1.75}
                aria-hidden="true"
              />
              <span className="flex min-w-0 flex-1 flex-col">
                <span className="truncate font-mono text-xs font-medium">{tool.name}</span>
                <span className="text-xs leading-snug text-muted-foreground">
                  {tool.description}
                </span>
              </span>
              <Button
                type="button"
                variant="ghost"
                size="icon-xs"
                className="shrink-0 opacity-0 transition-opacity group-hover:opacity-100 focus-visible:opacity-100"
                aria-label={`Remove ${tool.name}`}
                onClick={() => remove(tool.name)}
              >
                <XIcon />
              </Button>
            </li>
          ))}
        </ul>
      )}
    </div>
  )
}

function MarkdownField({
  id,
  value,
  onChange,
}: {
  id: string
  value: string
  onChange: (value: string) => void
}) {
  return (
    <Textarea
      id={id}
      rows={8}
      value={value}
      onChange={(event) => onChange(event.target.value)}
      className="font-mono text-xs"
    />
  )
}

function DefinitionEditor({
  definition,
  onChange,
}: {
  definition: AgentDefinitionView
  onChange: (definition: AgentDefinitionView) => void
}) {
  return (
    <div className="flex flex-col gap-4">
      <div className="flex flex-col gap-1.5">
        <label
          htmlFor="agent-role"
          className="text-xs font-medium tracking-wide text-muted-foreground"
        >
          Role
        </label>
        <Input
          id="agent-role"
          value={definition.role}
          onChange={(event) => onChange({ ...definition, role: event.target.value })}
        />
      </div>
      <div className="flex flex-col gap-1.5">
        <label
          htmlFor="agent-prompt"
          className="text-xs font-medium tracking-wide text-muted-foreground"
        >
          Prompt
        </label>
        <MarkdownField
          id="agent-prompt"
          value={definition.prompt}
          onChange={(prompt) => onChange({ ...definition, prompt })}
        />
      </div>
      <div className="flex flex-col gap-1.5">
        <div className="flex items-center justify-between">
          <span className="text-xs font-medium tracking-wide text-muted-foreground">Tools</span>
          <span className="text-xs text-muted-foreground">{definition.tools.length}</span>
        </div>
        <ToolsEditor
          tools={definition.tools}
          onChange={(tools) => onChange({ ...definition, tools })}
        />
      </div>
    </div>
  )
}

function InstanceDetails({ instance }: { instance: AgentInstance }) {
  const running = instance.state === "running"
  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <div className="flex flex-col gap-0.5 border-b border-border/60 px-4 py-3">
        <div className="flex items-center gap-2">
          <span
            className={cn(
              "size-2 shrink-0 rounded-full",
              running ? "animate-pulse bg-primary" : "bg-muted-foreground/50",
            )}
          />
          <FolderKanban
            className="size-3.5 shrink-0 text-muted-foreground"
            strokeWidth={1.75}
            aria-hidden="true"
          />
          <h3 className="truncate text-sm font-medium">{instance.projectName}</h3>
          <span
            className={cn(
              "ml-auto shrink-0 text-[10px] font-medium tracking-wide uppercase",
              running ? "text-primary" : "text-muted-foreground",
            )}
          >
            {instance.state}
          </span>
        </div>
        <p className="truncate pl-6 text-xs text-muted-foreground">
          {instance.session === null ? "No active session" : instance.session.task}
        </p>
      </div>
      {instance.session === null ? (
        <div className="flex flex-1 items-center justify-center text-xs text-muted-foreground">
          Idle
        </div>
      ) : (
        <SessionTranscript session={instance.session} />
      )}
    </div>
  )
}

export function AgentView({
  model,
  onSave,
  expanded = false,
}: {
  model: AgentConsoleModel
  onSave?: (definition: AgentDefinitionView) => void
  expanded?: boolean
}) {
  const [selectedId, setSelectedId] = useState<string | null>(null)
  const [definition, setDefinition] = useState<AgentDefinitionView>(model.definition)
  const dirty = useRef(false)

  useEffect(() => {
    dirty.current = false
    setDefinition(model.definition)
  }, [model.definition])

  const update = (next: AgentDefinitionView) => {
    dirty.current = true
    setDefinition(next)
  }

  useEffect(() => {
    if (!dirty.current || onSave === undefined) {
      return
    }
    const timer = setTimeout(() => {
      dirty.current = false
      onSave(definition)
    }, 600)
    return () => clearTimeout(timer)
  }, [definition, onSave])

  const nodes = useMemo(
    () =>
      buildNodes(model, definition).map((node) => ({
        ...node,
        selected: node.id === selectedId,
      })),
    [model, definition, selectedId],
  )
  const edges = useMemo(() => buildEdges(model, selectedId), [model, selectedId])

  const agentSelected = selectedId === AGENT_NODE_ID
  const selectedInstance =
    selectedId !== null && selectedId.startsWith(INSTANCE_PREFIX)
      ? (model.instances.find((instance) => `${INSTANCE_PREFIX}${instance.id}` === selectedId) ?? null)
      : null

  const title = agentSelected ? "Definition" : selectedInstance !== null ? "Instance" : "Details"

  return (
    <div className="relative h-full w-full">
      <ReactFlowProvider>
        <ReactFlow
          nodes={nodes}
          edges={edges}
          nodeTypes={nodeTypes}
          onNodeClick={(_event, node) => setSelectedId(node.id)}
          onPaneClick={() => setSelectedId(null)}
          nodesDraggable={false}
          nodesConnectable={false}
          elementsSelectable={false}
          fitView
          fitViewOptions={{ padding: 0.4 }}
          minZoom={0.4}
          maxZoom={1.5}
          colorMode="dark"
          proOptions={{ hideAttribution: true }}
        >
          <Background variant={BackgroundVariant.Dots} gap={22} size={1} />
        </ReactFlow>
      </ReactFlowProvider>
      {selectedId === null ? null : (
        <aside
          className={cn(
            "absolute inset-y-0 right-0 z-10 flex flex-col border-l border-border bg-card/95 backdrop-blur animate-in fade-in slide-in-from-right duration-150",
            expanded ? "w-[32rem]" : "w-80",
          )}
        >
          <header className="flex items-center gap-2 border-b border-border px-4 py-3">
            <h3 className="truncate text-sm font-medium">{title}</h3>
            <Button
              variant="ghost"
              size="icon-sm"
              className="ml-auto"
              aria-label="Close details"
              onClick={() => setSelectedId(null)}
            >
              <XIcon />
            </Button>
          </header>
          {agentSelected ? (
            <div className="min-h-0 flex-1 overflow-y-auto p-4">
              <DefinitionEditor definition={definition} onChange={update} />
            </div>
          ) : selectedInstance !== null ? (
            <InstanceDetails instance={selectedInstance} />
          ) : null}
        </aside>
      )}
    </div>
  )
}
