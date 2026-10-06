CREATE TABLE IF NOT EXISTS workspaces (
    slug TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    user TEXT NOT NULL,
    home TEXT NOT NULL,
    created_at TEXT NOT NULL
);
