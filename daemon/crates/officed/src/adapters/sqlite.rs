use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard};

use agent::{
    AgentDefinition, AgentId, AgentInstance, AgentInstanceRecord, AgentRepository, InstanceId,
    StoreError as AgentStoreError,
};
use project::{
    Project, ProjectId, ProjectRecord, ProjectRepository, StoreError as ProjectStoreError,
};
use rusqlite::{Connection, params};
use workspace::{
    StoreError as WorkspaceStoreError, Workspace, WorkspaceRecord, WorkspaceRepository,
};

const MIGRATIONS: &[(&str, &str)] = &[
    (
        "0001_workspaces",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../migrations/0001_workspaces.sql"
        )),
    ),
    (
        "0002_workspace_status",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../migrations/0002_workspace_status.sql"
        )),
    ),
    (
        "0003_agents",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../migrations/0003_agents.sql"
        )),
    ),
    (
        "0004_agent_definition_tools",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../migrations/0004_agent_definition_tools.sql"
        )),
    ),
    (
        "0005_agent_tool_ids",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../migrations/0005_agent_tool_ids.sql"
        )),
    ),
    (
        "0006_agent_definition_model",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../migrations/0006_agent_definition_model.sql"
        )),
    ),
    (
        "0007_projects",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../migrations/0007_projects.sql"
        )),
    ),
];

#[derive(Clone)]
pub struct SqliteStore {
    connection: Arc<Mutex<Connection>>,
}

impl SqliteStore {
    pub fn open(path: &Path) -> Result<Self, WorkspaceStoreError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|error| WorkspaceStoreError::Backend(Box::new(error)))?;
        }
        let connection = Connection::open(path).map_err(workspace_backend)?;
        Self::initialize(connection)
    }

    pub fn open_in_memory() -> Result<Self, WorkspaceStoreError> {
        let connection = Connection::open_in_memory().map_err(workspace_backend)?;
        Self::initialize(connection)
    }

    fn initialize(mut connection: Connection) -> Result<Self, WorkspaceStoreError> {
        connection
            .execute_batch("PRAGMA journal_mode = WAL;")
            .map_err(workspace_backend)?;
        apply_migrations(&mut connection)?;
        Ok(Self {
            connection: Arc::new(Mutex::new(connection)),
        })
    }

    fn connection(&self) -> Result<MutexGuard<'_, Connection>, ()> {
        self.connection.lock().map_err(|_| ())
    }
}

impl WorkspaceRepository for SqliteStore {
    fn save(&self, workspace: &Workspace) -> Result<(), WorkspaceStoreError> {
        self.connection()
            .map_err(|_| WorkspaceStoreError::Poisoned)?
            .execute(
                "INSERT INTO workspaces (slug, name, user, home, status, created_at)
                 VALUES (?1, ?2, ?3, ?4, 'active', datetime('now'))
                 ON CONFLICT(slug) DO UPDATE SET
                     name = excluded.name,
                     user = excluded.user,
                     home = excluded.home",
                params![
                    workspace.slug,
                    workspace.name,
                    workspace.user,
                    workspace.home
                ],
            )
            .map_err(workspace_backend)?;
        Ok(())
    }

    fn find(&self, slug: &str) -> Result<Option<WorkspaceRecord>, WorkspaceStoreError> {
        let connection = self
            .connection()
            .map_err(|_| WorkspaceStoreError::Poisoned)?;
        let mut statement = connection
            .prepare(
                "SELECT name, slug, user, home, created_at, status
                 FROM workspaces WHERE slug = ?1",
            )
            .map_err(workspace_backend)?;
        let mut rows = statement
            .query_map([slug], |row| {
                Ok(WorkspaceRecord {
                    name: row.get(0)?,
                    slug: row.get(1)?,
                    user: row.get(2)?,
                    home: row.get(3)?,
                    created_at: row.get(4)?,
                    status: row.get(5)?,
                })
            })
            .map_err(workspace_backend)?;

        match rows.next() {
            Some(row) => Ok(Some(row.map_err(workspace_backend)?)),
            None => Ok(None),
        }
    }

    fn list(&self) -> Result<Vec<WorkspaceRecord>, WorkspaceStoreError> {
        let connection = self
            .connection()
            .map_err(|_| WorkspaceStoreError::Poisoned)?;
        let mut statement = connection
            .prepare(
                "SELECT name, slug, user, home, created_at, status
                 FROM workspaces
                 ORDER BY created_at, slug",
            )
            .map_err(workspace_backend)?;
        let rows = statement
            .query_map([], |row| {
                Ok(WorkspaceRecord {
                    name: row.get(0)?,
                    slug: row.get(1)?,
                    user: row.get(2)?,
                    home: row.get(3)?,
                    created_at: row.get(4)?,
                    status: row.get(5)?,
                })
            })
            .map_err(workspace_backend)?;

        let mut records = Vec::new();
        for row in rows {
            records.push(row.map_err(workspace_backend)?);
        }
        Ok(records)
    }

    fn set_status(&self, slug: &str, status: &str) -> Result<(), WorkspaceStoreError> {
        self.connection()
            .map_err(|_| WorkspaceStoreError::Poisoned)?
            .execute(
                "UPDATE workspaces SET status = ?1 WHERE slug = ?2",
                params![status, slug],
            )
            .map_err(workspace_backend)?;
        Ok(())
    }

    fn remove(&self, slug: &str) -> Result<bool, WorkspaceStoreError> {
        let affected = self
            .connection()
            .map_err(|_| WorkspaceStoreError::Poisoned)?
            .execute("DELETE FROM workspaces WHERE slug = ?1", params![slug])
            .map_err(workspace_backend)?;
        Ok(affected > 0)
    }
}

impl AgentRepository for SqliteStore {
    fn save_definition(&self, definition: &AgentDefinition) -> Result<(), AgentStoreError> {
        let tools = serde_json::to_string(&definition.tools)
            .map_err(|error| AgentStoreError::Backend(Box::new(error)))?;
        self.connection()
            .map_err(|_| AgentStoreError::Poisoned)?
            .execute(
                "INSERT INTO agent_definitions
                     (id, name, role, prompt, tools, model, version, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, datetime('now'), datetime('now'))
                 ON CONFLICT(id) DO UPDATE SET
                     name = excluded.name,
                     role = excluded.role,
                     prompt = excluded.prompt,
                     tools = excluded.tools,
                     model = excluded.model,
                     version = excluded.version,
                     updated_at = datetime('now')",
                params![
                    definition.id.as_str(),
                    definition.name,
                    definition.role,
                    definition.prompt,
                    tools,
                    definition.model,
                    definition.version,
                ],
            )
            .map_err(agent_backend)?;
        Ok(())
    }

    fn find_definition(&self, id: &str) -> Result<Option<AgentDefinition>, AgentStoreError> {
        let connection = self.connection().map_err(|_| AgentStoreError::Poisoned)?;
        let mut statement = connection
            .prepare(
                "SELECT id, name, role, prompt, tools, model, version
                 FROM agent_definitions WHERE id = ?1",
            )
            .map_err(agent_backend)?;
        let mut rows = statement
            .query_map([id], definition_row)
            .map_err(agent_backend)?;

        match rows.next() {
            Some(row) => Ok(Some(row.map_err(agent_backend)??)),
            None => Ok(None),
        }
    }

    fn list_definitions(&self) -> Result<Vec<AgentDefinition>, AgentStoreError> {
        let connection = self.connection().map_err(|_| AgentStoreError::Poisoned)?;
        let mut statement = connection
            .prepare(
                "SELECT id, name, role, prompt, tools, model, version
                 FROM agent_definitions ORDER BY id",
            )
            .map_err(agent_backend)?;
        let rows = statement
            .query_map([], definition_row)
            .map_err(agent_backend)?;

        let mut definitions = Vec::new();
        for row in rows {
            definitions.push(row.map_err(agent_backend)??);
        }
        Ok(definitions)
    }

    fn remove_definition(&self, id: &str) -> Result<bool, AgentStoreError> {
        let affected = self
            .connection()
            .map_err(|_| AgentStoreError::Poisoned)?
            .execute("DELETE FROM agent_definitions WHERE id = ?1", params![id])
            .map_err(agent_backend)?;
        Ok(affected > 0)
    }

    fn save_instance(&self, instance: &AgentInstance) -> Result<(), AgentStoreError> {
        self.connection()
            .map_err(|_| AgentStoreError::Poisoned)?
            .execute(
                "INSERT INTO agent_instances (id, agent_id, workspace, project, created_at)
                 VALUES (?1, ?2, ?3, ?4, datetime('now'))",
                params![
                    instance.id.as_str(),
                    instance.agent.as_str(),
                    instance.workspace,
                    instance.project,
                ],
            )
            .map_err(agent_backend)?;
        Ok(())
    }

    fn find_instance(
        &self,
        agent: &str,
        workspace: &str,
        project: &str,
    ) -> Result<Option<AgentInstance>, AgentStoreError> {
        let connection = self.connection().map_err(|_| AgentStoreError::Poisoned)?;
        let mut statement = connection
            .prepare(
                "SELECT id, agent_id, workspace, project
                 FROM agent_instances
                 WHERE agent_id = ?1 AND workspace = ?2 AND project = ?3",
            )
            .map_err(agent_backend)?;
        let mut rows = statement
            .query_map(params![agent, workspace, project], instance_row)
            .map_err(agent_backend)?;

        match rows.next() {
            Some(row) => Ok(Some(row.map_err(agent_backend)?)),
            None => Ok(None),
        }
    }

    fn list_instances(
        &self,
        workspace: Option<&str>,
        project: Option<&str>,
    ) -> Result<Vec<AgentInstanceRecord>, AgentStoreError> {
        let connection = self.connection().map_err(|_| AgentStoreError::Poisoned)?;
        let mut statement = connection
            .prepare(
                "SELECT id, agent_id, workspace, project, created_at
                 FROM agent_instances
                 WHERE (?1 IS NULL OR workspace = ?1)
                   AND (?2 IS NULL OR project = ?2)
                 ORDER BY created_at, id",
            )
            .map_err(agent_backend)?;
        let rows = statement
            .query_map(params![workspace, project], instance_record_row)
            .map_err(agent_backend)?;

        let mut records = Vec::new();
        for row in rows {
            records.push(row.map_err(agent_backend)?);
        }
        Ok(records)
    }

    fn remove_instance(
        &self,
        agent: &str,
        workspace: &str,
        project: &str,
    ) -> Result<bool, AgentStoreError> {
        let affected = self
            .connection()
            .map_err(|_| AgentStoreError::Poisoned)?
            .execute(
                "DELETE FROM agent_instances
                 WHERE agent_id = ?1 AND workspace = ?2 AND project = ?3",
                params![agent, workspace, project],
            )
            .map_err(agent_backend)?;
        Ok(affected > 0)
    }
}

impl ProjectRepository for SqliteStore {
    fn save(&self, project: &Project) -> Result<(), ProjectStoreError> {
        self.connection()
            .map_err(|_| ProjectStoreError::Poisoned)?
            .execute(
                "INSERT INTO projects (id, workspace, name, created_at)
                 VALUES (?1, ?2, ?3, datetime('now'))",
                params![project.id.as_str(), project.workspace, project.name],
            )
            .map_err(project_backend)?;
        Ok(())
    }

    fn find(&self, workspace: &str, id: &str) -> Result<Option<ProjectRecord>, ProjectStoreError> {
        let connection = self.connection().map_err(|_| ProjectStoreError::Poisoned)?;
        let mut statement = connection
            .prepare(
                "SELECT id, workspace, name, created_at
                 FROM projects WHERE workspace = ?1 AND id = ?2",
            )
            .map_err(project_backend)?;
        let mut rows = statement
            .query_map(params![workspace, id], project_record_row)
            .map_err(project_backend)?;

        match rows.next() {
            Some(row) => Ok(Some(row.map_err(project_backend)?)),
            None => Ok(None),
        }
    }

    fn list(&self, workspace: Option<&str>) -> Result<Vec<ProjectRecord>, ProjectStoreError> {
        let connection = self.connection().map_err(|_| ProjectStoreError::Poisoned)?;
        let mut statement = connection
            .prepare(
                "SELECT id, workspace, name, created_at
                 FROM projects
                 WHERE (?1 IS NULL OR workspace = ?1)
                 ORDER BY created_at, id",
            )
            .map_err(project_backend)?;
        let rows = statement
            .query_map(params![workspace], project_record_row)
            .map_err(project_backend)?;

        let mut records = Vec::new();
        for row in rows {
            records.push(row.map_err(project_backend)?);
        }
        Ok(records)
    }

    fn remove(&self, workspace: &str, id: &str) -> Result<bool, ProjectStoreError> {
        let affected = self
            .connection()
            .map_err(|_| ProjectStoreError::Poisoned)?
            .execute(
                "DELETE FROM projects WHERE workspace = ?1 AND id = ?2",
                params![workspace, id],
            )
            .map_err(project_backend)?;
        Ok(affected > 0)
    }
}

fn definition_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<Result<AgentDefinition, AgentStoreError>> {
    let id: String = row.get(0)?;
    let name: String = row.get(1)?;
    let role: String = row.get(2)?;
    let prompt: String = row.get(3)?;
    let tools: String = row.get(4)?;
    let model: String = row.get(5)?;
    let version: i64 = row.get(6)?;

    let parsed = serde_json::from_str::<Vec<String>>(&tools)
        .map_err(|error| AgentStoreError::Backend(Box::new(error)));
    Ok(parsed.map(|tools| AgentDefinition {
        id: AgentId(id),
        name,
        role,
        prompt,
        tools,
        model,
        version: version as u32,
    }))
}

fn instance_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<AgentInstance> {
    Ok(AgentInstance {
        id: InstanceId(row.get(0)?),
        agent: AgentId(row.get(1)?),
        workspace: row.get(2)?,
        project: row.get(3)?,
    })
}

fn instance_record_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<AgentInstanceRecord> {
    Ok(AgentInstanceRecord {
        id: InstanceId(row.get(0)?),
        agent: AgentId(row.get(1)?),
        workspace: row.get(2)?,
        project: row.get(3)?,
        created_at: row.get(4)?,
    })
}

fn project_record_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ProjectRecord> {
    Ok(ProjectRecord {
        id: ProjectId(row.get(0)?),
        workspace: row.get(1)?,
        name: row.get(2)?,
        created_at: row.get(3)?,
    })
}

fn apply_migrations(connection: &mut Connection) -> Result<(), WorkspaceStoreError> {
    connection
        .execute_batch(
            "CREATE TABLE IF NOT EXISTS schema_migrations (
                 version TEXT PRIMARY KEY,
                 applied_at TEXT NOT NULL DEFAULT (datetime('now'))
             );",
        )
        .map_err(workspace_backend)?;

    for (version, sql) in MIGRATIONS {
        let applied: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM schema_migrations WHERE version = ?1",
                [version],
                |row| row.get(0),
            )
            .map_err(workspace_backend)?;
        if applied == 0 {
            let transaction = connection.transaction().map_err(workspace_backend)?;
            transaction.execute_batch(sql).map_err(workspace_backend)?;
            transaction
                .execute(
                    "INSERT INTO schema_migrations (version) VALUES (?1)",
                    [version],
                )
                .map_err(workspace_backend)?;
            transaction.commit().map_err(workspace_backend)?;
        }
    }

    Ok(())
}

fn workspace_backend(error: rusqlite::Error) -> WorkspaceStoreError {
    WorkspaceStoreError::Backend(Box::new(error))
}

fn agent_backend(error: rusqlite::Error) -> AgentStoreError {
    if let rusqlite::Error::SqliteFailure(sql_error, _) = &error
        && sql_error.code == rusqlite::ErrorCode::ConstraintViolation
    {
        return AgentStoreError::Conflict;
    }
    AgentStoreError::Backend(Box::new(error))
}

fn project_backend(error: rusqlite::Error) -> ProjectStoreError {
    if let rusqlite::Error::SqliteFailure(sql_error, _) = &error
        && sql_error.code == rusqlite::ErrorCode::ConstraintViolation
    {
        return ProjectStoreError::Conflict;
    }
    ProjectStoreError::Backend(Box::new(error))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn workspace(slug: &str) -> Workspace {
        Workspace {
            name: slug.to_owned(),
            slug: slug.to_owned(),
            user: slug.to_owned(),
            home: format!("/home/{slug}"),
        }
    }

    fn definition(id: &str, version: u32) -> AgentDefinition {
        AgentDefinition {
            id: AgentId(id.to_owned()),
            name: id.to_owned(),
            role: "backend".to_owned(),
            prompt: "prompt".to_owned(),
            tools: vec!["shell".to_owned(), "editor".to_owned()],
            model: "claude-sonnet".to_owned(),
            version,
        }
    }

    fn instance(agent: &str, workspace: &str, project: &str) -> AgentInstance {
        AgentInstance {
            id: InstanceId(format!("inst_{agent}_{project}")),
            agent: AgentId(agent.to_owned()),
            workspace: workspace.to_owned(),
            project: project.to_owned(),
        }
    }

    #[test]
    fn saves_and_lists_workspaces() {
        let store = SqliteStore::open_in_memory().expect("opens");
        WorkspaceRepository::save(&store, &workspace("acme")).expect("saves");

        assert_eq!(
            WorkspaceRepository::list(&store).expect("lists")[0].slug,
            "acme"
        );
    }

    #[test]
    fn finds_and_removes_workspaces() {
        let store = SqliteStore::open_in_memory().expect("opens");
        WorkspaceRepository::save(&store, &workspace("acme")).expect("saves");

        let found = WorkspaceRepository::find(&store, "acme")
            .expect("finds")
            .expect("present");
        assert_eq!(found.home, "/home/acme");
        assert!(
            WorkspaceRepository::find(&store, "ghost")
                .expect("finds")
                .is_none()
        );

        assert!(WorkspaceRepository::remove(&store, "acme").expect("removes"));
        assert!(!WorkspaceRepository::remove(&store, "acme").expect("removes"));
        assert!(WorkspaceRepository::list(&store).expect("lists").is_empty());
    }

    #[test]
    fn versions_definitions_by_id() {
        let store = SqliteStore::open_in_memory().expect("opens");
        store
            .save_definition(&definition("backend-engineer", 1))
            .expect("saves");
        store
            .save_definition(&definition("backend-engineer", 2))
            .expect("saves");

        let definitions = store.list_definitions().expect("lists");
        assert_eq!(definitions.len(), 1);
        assert_eq!(definitions[0].version, 2);
        assert_eq!(definitions[0].tools, vec!["shell", "editor"]);
        assert_eq!(definitions[0].model, "claude-sonnet");
    }

    #[test]
    fn enforces_unique_instance() {
        let store = SqliteStore::open_in_memory().expect("opens");
        store
            .save_instance(&instance("backend-engineer", "acme", "ecommerce"))
            .expect("saves");
        let error = store
            .save_instance(&instance("backend-engineer", "acme", "ecommerce"))
            .unwrap_err();
        assert!(matches!(error, AgentStoreError::Conflict));
    }

    #[test]
    fn lists_instances_with_filters() {
        let store = SqliteStore::open_in_memory().expect("opens");
        store
            .save_instance(&instance("backend-engineer", "acme", "ecommerce"))
            .expect("saves");
        store
            .save_instance(&instance("backend-engineer", "acme", "mobile"))
            .expect("saves");

        assert_eq!(store.list_instances(None, None).expect("lists").len(), 2);
        assert_eq!(
            store
                .list_instances(Some("acme"), Some("mobile"))
                .expect("lists")
                .len(),
            1
        );
        assert!(
            store
                .remove_instance("backend-engineer", "acme", "mobile")
                .expect("removes")
        );
        assert!(
            !store
                .remove_instance("backend-engineer", "acme", "mobile")
                .expect("removes")
        );
    }

    #[test]
    fn saves_lists_and_removes_projects() {
        let store = SqliteStore::open_in_memory().expect("opens");
        ProjectRepository::save(
            &store,
            &Project {
                id: ProjectId("ecommerce".to_owned()),
                workspace: "acme".to_owned(),
                name: "Ecommerce".to_owned(),
            },
        )
        .expect("saves");
        ProjectRepository::save(
            &store,
            &Project {
                id: ProjectId("mobile".to_owned()),
                workspace: "acme".to_owned(),
                name: "Mobile".to_owned(),
            },
        )
        .expect("saves");
        ProjectRepository::save(
            &store,
            &Project {
                id: ProjectId("ecommerce".to_owned()),
                workspace: "foo".to_owned(),
                name: "Ecommerce".to_owned(),
            },
        )
        .expect("saves");

        assert_eq!(
            ProjectRepository::list(&store, None).expect("lists").len(),
            3
        );
        assert_eq!(
            ProjectRepository::list(&store, Some("acme"))
                .expect("lists")
                .len(),
            2
        );
        let found = ProjectRepository::find(&store, "acme", "ecommerce")
            .expect("finds")
            .expect("present");
        assert_eq!(found.name, "Ecommerce");
        assert!(
            ProjectRepository::find(&store, "acme", "ghost")
                .expect("finds")
                .is_none()
        );

        assert!(ProjectRepository::remove(&store, "acme", "mobile").expect("removes"));
        assert!(!ProjectRepository::remove(&store, "acme", "mobile").expect("removes"));
        assert_eq!(
            ProjectRepository::list(&store, None).expect("lists").len(),
            2
        );
    }

    #[test]
    fn rejects_duplicate_project_in_same_workspace() {
        let store = SqliteStore::open_in_memory().expect("opens");
        let project = Project {
            id: ProjectId("ecommerce".to_owned()),
            workspace: "acme".to_owned(),
            name: "Ecommerce".to_owned(),
        };
        ProjectRepository::save(&store, &project).expect("saves");
        let error = ProjectRepository::save(&store, &project).unwrap_err();
        assert!(matches!(error, ProjectStoreError::Conflict));
    }
}
