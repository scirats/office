CREATE TABLE IF NOT EXISTS agent_definitions (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    role TEXT NOT NULL,
    prompt TEXT NOT NULL,
    runtime TEXT NOT NULL,
    policy TEXT NOT NULL,
    skills TEXT NOT NULL,
    version INTEGER NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS agent_instances (
    id TEXT PRIMARY KEY,
    agent_id TEXT NOT NULL,
    workspace TEXT NOT NULL,
    project TEXT NOT NULL,
    created_at TEXT NOT NULL,
    UNIQUE (agent_id, workspace, project)
);
