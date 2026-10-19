CREATE TABLE IF NOT EXISTS projects (
    id TEXT NOT NULL,
    workspace TEXT NOT NULL,
    name TEXT NOT NULL,
    created_at TEXT NOT NULL,
    PRIMARY KEY (workspace, id)
);
