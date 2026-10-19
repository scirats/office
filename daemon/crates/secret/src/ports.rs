use crate::domain::Secret;

#[derive(Debug, thiserror::Error)]
pub enum SecretError {
    #[error("secret store is read-only")]
    ReadOnly,
    #[error("could not persist secret: {0}")]
    Persist(String),
    #[error("could not reload secret store: {0}")]
    Reload(String),
}

pub trait SecretStore: Send + Sync {
    fn resolve(&self, reference: &str) -> Option<Secret>;

    fn references(&self) -> Vec<String>;

    fn set(&self, reference: &str, value: &str) -> Result<(), SecretError>;

    fn reload(&self) -> Result<(), SecretError>;
}
