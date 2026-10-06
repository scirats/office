use crate::domain::WorkspaceObservation;

pub trait WorkspaceInspector: Send + Sync {
    fn inspect(&self, user: &str, home: &str) -> Result<WorkspaceObservation, InspectError>;
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum InspectError {
    #[error("inspection is not supported on this platform")]
    Unsupported,
}
