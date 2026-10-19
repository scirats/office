use crate::domain::{
    AgentDefinition, AgentId, AgentInstance, AgentInstanceRecord, InstanceId, is_valid_slug,
    slugify,
};
use crate::ports::{AgentRepository, InstanceIdSource, StoreError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefineAgent {
    pub name: String,
    pub role: String,
    pub prompt: String,
    pub tools: Vec<String>,
    pub model: String,
}

impl DefineAgent {
    pub fn new(
        name: impl Into<String>,
        role: impl Into<String>,
        prompt: impl Into<String>,
        tools: Vec<String>,
        model: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            role: role.into(),
            prompt: prompt.into(),
            tools,
            model: model.into(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AgentError {
    #[error("agent name does not yield a valid id")]
    InvalidName,
    #[error("agent role is empty")]
    InvalidRole,
    #[error("tool name is empty")]
    InvalidTool,
    #[error("model id is empty")]
    InvalidModel,
    #[error("workspace or project is not a valid slug")]
    InvalidScope,
    #[error("agent definition not found: {0}")]
    UnknownAgent(String),
    #[error("agent definition is still linked to {0} project(s)")]
    StillLinked(usize),
    #[error("agent is already linked to this project")]
    AlreadyLinked,
    #[error(transparent)]
    Store(StoreError),
}

fn has_duplicates(values: &[String]) -> bool {
    values
        .iter()
        .enumerate()
        .any(|(index, value)| values[..index].contains(value))
}

impl From<StoreError> for AgentError {
    fn from(error: StoreError) -> Self {
        match error {
            StoreError::Conflict => AgentError::AlreadyLinked,
            other => AgentError::Store(other),
        }
    }
}

pub struct AgentService {
    repository: Box<dyn AgentRepository>,
    ids: Box<dyn InstanceIdSource>,
}

impl AgentService {
    pub fn new(repository: Box<dyn AgentRepository>, ids: Box<dyn InstanceIdSource>) -> Self {
        Self { repository, ids }
    }

    pub fn define(&self, input: DefineAgent) -> Result<AgentDefinition, AgentError> {
        let id = slugify(&input.name);
        if !is_valid_slug(&id) {
            return Err(AgentError::InvalidName);
        }
        if input.role.trim().is_empty() {
            return Err(AgentError::InvalidRole);
        }
        if input.tools.iter().any(|id| id.trim().is_empty()) || has_duplicates(&input.tools) {
            return Err(AgentError::InvalidTool);
        }
        if input.model.trim().is_empty() {
            return Err(AgentError::InvalidModel);
        }

        let version = self
            .repository
            .find_definition(&id)?
            .map(|current| current.version + 1)
            .unwrap_or(1);

        let definition = AgentDefinition {
            id: AgentId(id),
            name: input.name.trim().to_owned(),
            role: input.role.trim().to_owned(),
            prompt: input.prompt,
            tools: input.tools,
            model: input.model.trim().to_owned(),
            version,
        };

        self.repository.save_definition(&definition)?;
        Ok(definition)
    }

    pub fn list_definitions(&self) -> Result<Vec<AgentDefinition>, AgentError> {
        Ok(self.repository.list_definitions()?)
    }

    pub fn delete_definition(&self, id: &str) -> Result<String, AgentError> {
        let definition = self
            .repository
            .find_definition(id)?
            .ok_or_else(|| AgentError::UnknownAgent(id.to_owned()))?;
        let linked = self
            .repository
            .list_instances(None, None)?
            .into_iter()
            .filter(|instance| instance.agent.as_str() == id)
            .count();
        if linked > 0 {
            return Err(AgentError::StillLinked(linked));
        }
        self.repository.remove_definition(id)?;
        Ok(definition.id.0)
    }

    pub fn link(
        &self,
        agent: &str,
        workspace: &str,
        project: &str,
    ) -> Result<AgentInstance, AgentError> {
        if !is_valid_slug(agent) {
            return Err(AgentError::InvalidName);
        }
        if !is_valid_slug(workspace) || !is_valid_slug(project) {
            return Err(AgentError::InvalidScope);
        }
        if self.repository.find_definition(agent)?.is_none() {
            return Err(AgentError::UnknownAgent(agent.to_owned()));
        }

        let instance = AgentInstance {
            id: InstanceId(self.ids.instance_id()),
            agent: AgentId(agent.to_owned()),
            workspace: workspace.to_owned(),
            project: project.to_owned(),
        };

        self.repository.save_instance(&instance)?;
        Ok(instance)
    }

    pub fn list_instances(
        &self,
        workspace: Option<&str>,
        project: Option<&str>,
    ) -> Result<Vec<AgentInstanceRecord>, AgentError> {
        Ok(self.repository.list_instances(workspace, project)?)
    }

    pub fn unlink(
        &self,
        agent: &str,
        workspace: &str,
        project: &str,
    ) -> Result<Option<String>, AgentError> {
        let Some(instance) = self.repository.find_instance(agent, workspace, project)? else {
            return Ok(None);
        };
        self.repository.remove_instance(agent, workspace, project)?;
        Ok(Some(instance.id.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    #[derive(Default)]
    struct FakeRepository {
        definitions: Mutex<Vec<AgentDefinition>>,
        instances: Mutex<Vec<AgentInstanceRecord>>,
    }

    impl AgentRepository for FakeRepository {
        fn save_definition(&self, definition: &AgentDefinition) -> Result<(), StoreError> {
            let mut definitions = self.definitions.lock().unwrap();
            match definitions.iter_mut().find(|item| item.id == definition.id) {
                Some(existing) => *existing = definition.clone(),
                None => definitions.push(definition.clone()),
            }
            Ok(())
        }

        fn find_definition(&self, id: &str) -> Result<Option<AgentDefinition>, StoreError> {
            Ok(self
                .definitions
                .lock()
                .unwrap()
                .iter()
                .find(|item| item.id.as_str() == id)
                .cloned())
        }

        fn list_definitions(&self) -> Result<Vec<AgentDefinition>, StoreError> {
            Ok(self.definitions.lock().unwrap().clone())
        }

        fn remove_definition(&self, id: &str) -> Result<bool, StoreError> {
            let mut definitions = self.definitions.lock().unwrap();
            let before = definitions.len();
            definitions.retain(|item| item.id.as_str() != id);
            Ok(definitions.len() != before)
        }

        fn save_instance(&self, instance: &AgentInstance) -> Result<(), StoreError> {
            let mut instances = self.instances.lock().unwrap();
            let duplicate = instances.iter().any(|item| {
                item.agent == instance.agent
                    && item.workspace == instance.workspace
                    && item.project == instance.project
            });
            if duplicate {
                return Err(StoreError::Conflict);
            }
            instances.push(AgentInstanceRecord {
                id: instance.id.clone(),
                agent: instance.agent.clone(),
                workspace: instance.workspace.clone(),
                project: instance.project.clone(),
                created_at: "2026-01-01T00:00:00Z".to_owned(),
            });
            Ok(())
        }

        fn find_instance(
            &self,
            agent: &str,
            workspace: &str,
            project: &str,
        ) -> Result<Option<AgentInstance>, StoreError> {
            Ok(self
                .instances
                .lock()
                .unwrap()
                .iter()
                .find(|item| {
                    item.agent.as_str() == agent
                        && item.workspace == workspace
                        && item.project == project
                })
                .map(|item| AgentInstance {
                    id: item.id.clone(),
                    agent: item.agent.clone(),
                    workspace: item.workspace.clone(),
                    project: item.project.clone(),
                }))
        }

        fn list_instances(
            &self,
            workspace: Option<&str>,
            project: Option<&str>,
        ) -> Result<Vec<AgentInstanceRecord>, StoreError> {
            Ok(self
                .instances
                .lock()
                .unwrap()
                .iter()
                .filter(|item| workspace.is_none_or(|value| item.workspace == value))
                .filter(|item| project.is_none_or(|value| item.project == value))
                .cloned()
                .collect())
        }

        fn remove_instance(
            &self,
            agent: &str,
            workspace: &str,
            project: &str,
        ) -> Result<bool, StoreError> {
            let mut instances = self.instances.lock().unwrap();
            let before = instances.len();
            instances.retain(|item| {
                !(item.agent.as_str() == agent
                    && item.workspace == workspace
                    && item.project == project)
            });
            Ok(instances.len() != before)
        }
    }

    #[derive(Default)]
    struct SequentialIds(Mutex<u64>);

    impl InstanceIdSource for SequentialIds {
        fn instance_id(&self) -> String {
            let mut next = self.0.lock().unwrap();
            *next += 1;
            format!("inst_{next}")
        }
    }

    fn service() -> AgentService {
        AgentService::new(
            Box::new(FakeRepository::default()),
            Box::new(SequentialIds::default()),
        )
    }

    fn define(service: &AgentService) -> AgentDefinition {
        let tools = vec!["shell".to_owned()];
        service
            .define(DefineAgent::new(
                "Backend Engineer",
                "backend",
                "system prompt",
                tools,
                "claude-sonnet",
            ))
            .expect("defines")
    }

    #[test]
    fn defines_with_slug_and_version() {
        let service = service();
        let definition = define(&service);
        assert_eq!(definition.id.as_str(), "backend-engineer");
        assert_eq!(definition.name, "Backend Engineer");
        assert_eq!(definition.role, "backend");
        assert_eq!(definition.version, 1);

        let updated = define(&service);
        assert_eq!(updated.version, 2);
        assert_eq!(service.list_definitions().unwrap().len(), 1);
    }

    #[test]
    fn rejects_invalid_name_and_role() {
        let service = service();
        assert!(matches!(
            service.define(DefineAgent::new("!!!", "backend", "p", vec![], "claude")),
            Err(AgentError::InvalidName)
        ));
        assert!(matches!(
            service.define(DefineAgent::new("Agent", "", "p", vec![], "claude")),
            Err(AgentError::InvalidRole)
        ));
        assert!(matches!(
            service.define(DefineAgent::new(
                "Agent",
                "backend",
                "p",
                vec!["  ".to_owned()],
                "claude"
            )),
            Err(AgentError::InvalidTool)
        ));
        assert!(matches!(
            service.define(DefineAgent::new(
                "Agent",
                "backend",
                "p",
                vec!["shell".to_owned(), "shell".to_owned()],
                "claude"
            )),
            Err(AgentError::InvalidTool)
        ));
    }

    #[test]
    fn rejects_empty_model() {
        let service = service();
        assert!(matches!(
            service.define(DefineAgent::new("Agent", "backend", "p", vec![], "  ")),
            Err(AgentError::InvalidModel)
        ));
    }

    #[test]
    fn links_only_existing_agents() {
        let service = service();
        assert!(matches!(
            service.link("backend-engineer", "acme", "ecommerce"),
            Err(AgentError::UnknownAgent(id)) if id == "backend-engineer"
        ));

        define(&service);
        let instance = service
            .link("backend-engineer", "acme", "ecommerce")
            .expect("links");
        assert_eq!(instance.agent.as_str(), "backend-engineer");
        assert_eq!(instance.workspace, "acme");
        assert_eq!(instance.project, "ecommerce");
    }

    #[test]
    fn rejects_duplicate_instance() {
        let service = service();
        define(&service);
        service
            .link("backend-engineer", "acme", "ecommerce")
            .expect("links");

        assert!(matches!(
            service.link("backend-engineer", "acme", "ecommerce"),
            Err(AgentError::AlreadyLinked)
        ));
    }

    #[test]
    fn rejects_invalid_scope() {
        let service = service();
        define(&service);
        assert!(matches!(
            service.link("backend-engineer", "Acme", "ecommerce"),
            Err(AgentError::InvalidScope)
        ));
    }

    #[test]
    fn lists_and_unlinks_instances() {
        let service = service();
        define(&service);
        service
            .link("backend-engineer", "acme", "ecommerce")
            .expect("links");
        service
            .link("backend-engineer", "acme", "mobile")
            .expect("links");

        assert_eq!(service.list_instances(Some("acme"), None).unwrap().len(), 2);
        assert_eq!(
            service
                .list_instances(Some("acme"), Some("mobile"))
                .unwrap()
                .len(),
            1
        );

        assert_eq!(
            service
                .unlink("backend-engineer", "acme", "mobile")
                .expect("unlinks"),
            Some("inst_2".to_owned())
        );
        assert_eq!(
            service
                .unlink("backend-engineer", "acme", "mobile")
                .expect("idempotent"),
            None
        );
        assert_eq!(service.list_instances(None, None).unwrap().len(), 1);
    }

    #[test]
    fn deletes_only_unlinked_definitions() {
        let service = service();
        assert!(matches!(
            service.delete_definition("ghost"),
            Err(AgentError::UnknownAgent(id)) if id == "ghost"
        ));

        define(&service);
        service
            .link("backend-engineer", "acme", "ecommerce")
            .expect("links");
        assert!(matches!(
            service.delete_definition("backend-engineer"),
            Err(AgentError::StillLinked(1))
        ));

        service
            .unlink("backend-engineer", "acme", "ecommerce")
            .expect("unlinks");
        assert_eq!(
            service.delete_definition("backend-engineer").unwrap(),
            "backend-engineer"
        );
        assert!(service.list_definitions().unwrap().is_empty());
    }
}
