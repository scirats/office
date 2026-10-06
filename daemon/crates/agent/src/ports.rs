use crate::domain::{AgentDefinition, AgentInstance, AgentInstanceRecord};

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("store operation failed")]
    Backend(#[source] Box<dyn std::error::Error + Send + Sync>),
    #[error("store state is poisoned")]
    Poisoned,
    #[error("store constraint violated")]
    Conflict,
}

pub trait AgentRepository: Send + Sync {
    fn save_definition(&self, definition: &AgentDefinition) -> Result<(), StoreError>;
    fn find_definition(&self, id: &str) -> Result<Option<AgentDefinition>, StoreError>;
    fn list_definitions(&self) -> Result<Vec<AgentDefinition>, StoreError>;

    fn save_instance(&self, instance: &AgentInstance) -> Result<(), StoreError>;
    fn find_instance(
        &self,
        agent: &str,
        workspace: &str,
        project: &str,
    ) -> Result<Option<AgentInstance>, StoreError>;
    fn list_instances(
        &self,
        workspace: Option<&str>,
        project: Option<&str>,
    ) -> Result<Vec<AgentInstanceRecord>, StoreError>;
    fn remove_instance(
        &self,
        agent: &str,
        workspace: &str,
        project: &str,
    ) -> Result<bool, StoreError>;
}

pub trait InstanceIdSource: Send + Sync {
    fn instance_id(&self) -> String;
}
