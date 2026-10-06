use crate::domain::{Workspace, WorkspaceRecord};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Provisioned {
    pub user: String,
    pub home: String,
}

pub trait UserProvisioner: Send + Sync {
    fn provision(&self, slug: &str) -> Result<Provisioned, ProvisionError>;
}

pub trait WorkspaceRepository: Send + Sync {
    fn save(&self, workspace: &Workspace) -> Result<(), StoreError>;
    fn list(&self) -> Result<Vec<WorkspaceRecord>, StoreError>;
    fn set_status(&self, slug: &str, status: &str) -> Result<(), StoreError>;
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ProvisionError {
    #[error("user provisioning is not supported on this platform")]
    Unsupported,
    #[error("{action} failed for {target}: {message}")]
    Command {
        action: String,
        target: String,
        message: String,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("store operation failed")]
    Backend(#[source] Box<dyn std::error::Error + Send + Sync>),
    #[error("store state is poisoned")]
    Poisoned,
}
