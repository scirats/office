import type { ProjectDetail, TaskColumn, TaskPriority } from "./project"

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

export function buildSampleProjectBoard(project: ProjectDetail): TaskColumn[] {
  let sequence = 1000
  return COLUMNS.map((column) => {
    const cards = SEEDS.flatMap((seed, index) => {
      if (seed.column !== column.id) {
        return []
      }
      sequence += 1
      const assignee =
        project.agents.length === 0
          ? "unassigned"
          : (project.agents[index % project.agents.length]?.name ?? "unassigned")
      return [
        {
          id: `${project.id}:${sequence}`,
          title: seed.title,
          priority: seed.priority,
          agent: assignee,
          execution: `exec_${sequence}`,
        },
      ]
    })
    return { id: column.id, title: column.title, cards }
  })
}
