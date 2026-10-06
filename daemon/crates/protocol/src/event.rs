use serde::{Deserialize, Serialize};

use crate::tool::ToolView;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandAccepted {
    pub command_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandRejected {
    pub command_id: String,
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskAccepted {
    pub task_id: String,
    pub execution_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionStateChanged {
    pub execution_id: String,
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionLog {
    pub execution_id: String,
    pub stream: String,
    pub line: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OfficeError {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceCreated {
    pub name: String,
    pub slug: String,
    pub user: String,
    pub home: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceSummary {
    pub name: String,
    pub slug: String,
    pub user: String,
    pub home: String,
    pub created_at: String,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceListed {
    pub workspaces: Vec<WorkspaceSummary>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceStatusChanged {
    pub slug: String,
    pub status: String,
    pub inconsistencies: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentDefinitionView {
    pub id: String,
    pub name: String,
    pub role: String,
    pub prompt: String,
    pub tools: Vec<ToolView>,
    pub version: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentDefinitionListed {
    pub definitions: Vec<AgentDefinitionView>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentInstanceView {
    pub id: String,
    pub agent: String,
    pub workspace: String,
    pub project: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentInstanceSummary {
    pub id: String,
    pub agent: String,
    pub workspace: String,
    pub project: String,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentInstanceListed {
    pub instances: Vec<AgentInstanceSummary>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentInstanceDeleted {
    pub agent: String,
    pub workspace: String,
    pub project: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolListed {
    pub tools: Vec<ToolView>,
}

pub const CODE_RESYNC_REQUIRED: &str = "resync_required";
pub const CODE_INVALID_EVENT: &str = "invalid_event";
pub const CODE_UNKNOWN_COMMAND: &str = "unknown_command";
pub const CODE_COMMAND_FAILED: &str = "command_failed";
