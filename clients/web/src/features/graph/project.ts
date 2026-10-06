export interface ProjectAgent {
  id: string
  name: string
  role: string
}

export interface ProjectDetail {
  id: string
  name: string
  agents: ProjectAgent[]
}

export type TaskPriority = "low" | "medium" | "high"

export interface TaskCard {
  id: string
  title: string
  priority: TaskPriority
  agent: string
  execution: string
}

export interface TaskColumn {
  id: string
  title: string
  cards: TaskCard[]
}

const COLUMNS: { id: string; title: string }[] = [
  { id: "backlog", title: "Backlog" },
  { id: "queued", title: "Queued" },
  { id: "running", title: "Running" },
  { id: "review", title: "Review" },
  { id: "done", title: "Done" },
]

const SEEDS: { column: string; title: string; priority: TaskPriority }[] = [
  { column: "backlog", title: "Define project knowledge layout", priority: "low" },
  { column: "backlog", title: "Split monolith repositories", priority: "medium" },
  { column: "queued", title: "Add execution timeout policy", priority: "medium" },
  { column: "running", title: "Migrate CI to git worktrees", priority: "high" },
  { column: "review", title: "Harden sandbox profile", priority: "high" },
  { column: "done", title: "Bootstrap company workspace", priority: "low" },
]

export function buildProjectBoard(project: ProjectDetail): TaskColumn[] {
  let sequence = 1000
  return COLUMNS.map((column) => {
    const cards = SEEDS.flatMap((seed, index) => {
      if (seed.column !== column.id) {
        return []
      }
      sequence += 1
      const agent = project.agents[index % project.agents.length]
      return [
        {
          id: `${project.id}:${sequence}`,
          title: seed.title,
          priority: seed.priority,
          agent: agent?.name ?? "unassigned",
          execution: `exec_${sequence}`,
        },
      ]
    })
    return { id: column.id, title: column.title, cards }
  })
}
