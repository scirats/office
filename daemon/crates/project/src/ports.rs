use crate::domain::{Project, ProjectRecord};

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("store operation failed")]
    Backend(#[source] Box<dyn std::error::Error + Send + Sync>),
    #[error("store state is poisoned")]
    Poisoned,
    #[error("store constraint violated")]
    Conflict,
}

pub trait ProjectRepository: Send + Sync {
    fn save(&self, project: &Project) -> Result<(), StoreError>;
    fn find(&self, workspace: &str, id: &str) -> Result<Option<ProjectRecord>, StoreError>;
    fn list(&self, workspace: Option<&str>) -> Result<Vec<ProjectRecord>, StoreError>;
    fn remove(&self, workspace: &str, id: &str) -> Result<bool, StoreError>;
}
