#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ConnectionId(pub String);

impl ConnectionId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Filter {
    pub subject_prefix: Option<String>,
}

impl Filter {
    pub fn any() -> Self {
        Self {
            subject_prefix: None,
        }
    }

    pub fn prefix(prefix: impl Into<String>) -> Self {
        Self {
            subject_prefix: Some(prefix.into()),
        }
    }

    pub fn matches(&self, subject: &str) -> bool {
        match &self.subject_prefix {
            Some(prefix) => subject.starts_with(prefix),
            None => true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredEvent {
    pub sequence: u64,
    pub subject: String,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Publish {
    pub sequence: u64,
    pub recipients: Vec<ConnectionId>,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ReplayError {
    #[error("resync required for {subject}: oldest retained sequence is {oldest}")]
    ResyncRequired { subject: String, oldest: u64 },
}
