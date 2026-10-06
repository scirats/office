export interface AgentProject {
  id: string
  name: string
}

export interface AgentDetail {
  id: string
  name: string
  role: string
  projects: AgentProject[]
}

export type AgentSessionState = "BOOTSTRAPPING" | "RUNNING"

export type TranscriptKind = "cmd" | "out" | "ok" | "edit" | "warn" | "meta"

export interface TranscriptLine {
  kind: TranscriptKind
  text: string
}

export interface AgentSession {
  id: string
  task: string
  state: AgentSessionState
  elapsed: string
  resources: { cpu: string; memory: string }
  transcript: TranscriptLine[]
}

export interface AgentInstance {
  id: string
  projectName: string
  state: "running" | "idle"
  session: AgentSession | null
  queue: number
}

export interface AgentTool {
  name: string
  description: string
}

export interface AgentDefinitionView {
  name: string
  role: string
  prompt: string
  tools: AgentTool[]
}

export interface AgentConsoleModel {
  definition: AgentDefinitionView
  instances: AgentInstance[]
}

const TOOL_CATALOG: Record<string, string> = {
  shell: "Run commands inside the workspace homedir",
  editor: "Read and modify project files",
  tests: "Run the project test suite",
  database: "Inspect and query the project database",
  browser: "Fetch and render web pages",
  scanner: "Run security scanners against the project",
  network: "Make restricted network requests",
  secrets: "Read injected short-lived secrets",
  diff: "Produce and inspect code diffs",
  "static-analysis": "Run static analysis checks",
  adr: "Read and write architecture decisions",
}

export const AGENT_TOOL_CATALOG: AgentTool[] = Object.entries(TOOL_CATALOG).map(
  ([name, description]) => ({ name, description }),
)

const ROLE_TOOL_NAMES: Record<string, string[]> = {
  backend: ["shell", "editor", "tests", "database"],
  frontend: ["shell", "editor", "browser", "tests"],
  security: ["shell", "scanner", "network", "secrets"],
  reviewer: ["shell", "diff", "static-analysis"],
  architect: ["shell", "editor", "adr"],
}

const TASK_SEEDS = [
  "Implement idempotency keys",
  "Harden sandbox profile",
  "Migrate CI to git worktrees",
  "Review dependency updates",
]

function toolsFor(role: string): AgentTool[] {
  const names = ROLE_TOOL_NAMES[role] ?? ["shell", "editor"]
  return names.map((name) => ({
    name,
    description: TOOL_CATALOG[name] ?? "",
  }))
}

function sessionFor(project: AgentProject, index: number): AgentSession {
  const sequence = 1004 + index
  return {
    id: `sess_${sequence}`,
    task: TASK_SEEDS[index % TASK_SEEDS.length],
    state: "RUNNING",
    elapsed: "12.4s",
    resources: { cpu: "1.25 cores", memory: "412 MiB" },
    transcript: [
      { kind: "meta", text: `session sess_${sequence} · project ${project.name}` },
      { kind: "cmd", text: "$ git status" },
      { kind: "out", text: "On branch task/office" },
      { kind: "ok", text: "clean worktree" },
      { kind: "cmd", text: "$ pytest -q" },
      { kind: "out", text: "42 passed in 3.21s" },
      { kind: "edit", text: `edit src/${project.name}/service.ts  (+18 −4)` },
      { kind: "warn", text: "network egress blocked (policy: restricted-egress)" },
      { kind: "meta", text: "exec_1004 running · 12.4s" },
    ],
  }
}

export function buildAgentConsole(detail: AgentDetail): AgentConsoleModel {
  const instances = detail.projects.map((project, index) => {
    const running = index === 0
    return {
      id: `inst_${detail.id}:${project.id}`,
      projectName: project.name,
      state: running ? "running" : "idle",
      session: running ? sessionFor(project, index) : null,
      queue: running ? 0 : index,
    } satisfies AgentInstance
  })
  return {
    definition: {
      name: detail.name,
      role: detail.role,
      prompt: `You are ${detail.name}, a ${detail.role} specialist. Work only on the assigned task and report the result.`,
      tools: toolsFor(detail.role),
    },
    instances,
  }
}
