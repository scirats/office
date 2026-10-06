//! Office v1 wire vocabulary.

/// WebSocket subprotocol negotiated in the HTTP upgrade handshake.
pub const SUBPROTOCOL: &str = "office.v1";

/// Message types sent by clients towards the daemon.
pub mod command {
    pub const SUBSCRIPTION_ADD: &str = "office.subscription.add.v1";
    pub const SUBSCRIPTION_REMOVE: &str = "office.subscription.remove.v1";
    pub const SUBSCRIPTION_RESUME: &str = "office.subscription.resume.v1";
    pub const TASK_SUBMIT: &str = "office.task.submit.v1";
    pub const EXECUTION_CANCEL: &str = "office.execution.cancel.v1";
    pub const WORKSPACE_CREATE: &str = "office.workspace.create.v1";
    pub const WORKSPACE_LIST: &str = "office.workspace.list.v1";
    pub const AGENT_DEFINITION_CREATE: &str = "office.agent.definition.create.v1";
    pub const AGENT_DEFINITION_LIST: &str = "office.agent.definition.list.v1";
    pub const AGENT_INSTANCE_CREATE: &str = "office.agent.instance.create.v1";
    pub const AGENT_INSTANCE_LIST: &str = "office.agent.instance.list.v1";
    pub const AGENT_INSTANCE_DELETE: &str = "office.agent.instance.delete.v1";
    pub const TOOL_LIST: &str = "office.tool.list.v1";
}

/// Message types sent by the daemon towards clients.
pub mod event {
    pub const COMMAND_ACCEPTED: &str = "office.command.accepted.v1";
    pub const COMMAND_REJECTED: &str = "office.command.rejected.v1";
    pub const TASK_ACCEPTED: &str = "office.task.accepted.v1";
    pub const EXECUTION_STATE_CHANGED: &str = "office.execution.state_changed.v1";
    pub const EXECUTION_LOG: &str = "office.execution.log.v1";
    pub const WORKSPACE_CREATED: &str = "office.workspace.created.v1";
    pub const WORKSPACE_LISTED: &str = "office.workspace.listed.v1";
    pub const WORKSPACE_STATUS_CHANGED: &str = "office.workspace.status_changed.v1";
    pub const AGENT_DEFINITION_CREATED: &str = "office.agent.definition.created.v1";
    pub const AGENT_DEFINITION_LISTED: &str = "office.agent.definition.listed.v1";
    pub const AGENT_INSTANCE_CREATED: &str = "office.agent.instance.created.v1";
    pub const AGENT_INSTANCE_LISTED: &str = "office.agent.instance.listed.v1";
    pub const AGENT_INSTANCE_DELETED: &str = "office.agent.instance.deleted.v1";
    pub const TOOL_LISTED: &str = "office.tool.listed.v1";
    pub const ERROR: &str = "office.error.v1";
}
