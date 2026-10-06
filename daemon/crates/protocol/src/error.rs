#[derive(Debug, thiserror::Error)]
pub enum ProtocolError {
    #[error("invalid cloud event json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("unsupported cloud events specversion: {0}")]
    UnsupportedSpecVersion(String),
    #[error("unknown command type: {0}")]
    UnknownCommand(String),
}
