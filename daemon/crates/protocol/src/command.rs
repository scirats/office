use serde::{Deserialize, Serialize};

use crate::CloudEvent;
use crate::error::ProtocolError;
use crate::types;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Filter {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subject_prefix: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubscriptionAdd {
    pub filter: Filter,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubscriptionRemove {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filter: Option<Filter>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubscriptionResume {
    pub subject: String,
    pub last_sequence: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskSubmit {
    pub company: String,
    pub project: String,
    pub agent: String,
    pub input: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionCancel {
    pub execution_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceCreate {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceList {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentDefinitionCreate {
    pub name: String,
    pub role: String,
    #[serde(default)]
    pub prompt: String,
    #[serde(default)]
    pub tools: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolList {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentDefinitionList {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentInstanceCreate {
    pub agent: String,
    pub workspace: String,
    pub project: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentInstanceList {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workspace: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentInstanceDelete {
    pub agent: String,
    pub workspace: String,
    pub project: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    SubscriptionAdd(SubscriptionAdd),
    SubscriptionRemove(SubscriptionRemove),
    SubscriptionResume(SubscriptionResume),
    TaskSubmit(TaskSubmit),
    ExecutionCancel(ExecutionCancel),
    WorkspaceCreate(WorkspaceCreate),
    WorkspaceList(WorkspaceList),
    AgentDefinitionCreate(AgentDefinitionCreate),
    AgentDefinitionList(AgentDefinitionList),
    AgentInstanceCreate(AgentInstanceCreate),
    AgentInstanceList(AgentInstanceList),
    AgentInstanceDelete(AgentInstanceDelete),
    ToolList(ToolList),
}

impl Command {
    pub fn type_name(&self) -> &'static str {
        match self {
            Command::SubscriptionAdd(_) => types::command::SUBSCRIPTION_ADD,
            Command::SubscriptionRemove(_) => types::command::SUBSCRIPTION_REMOVE,
            Command::SubscriptionResume(_) => types::command::SUBSCRIPTION_RESUME,
            Command::TaskSubmit(_) => types::command::TASK_SUBMIT,
            Command::ExecutionCancel(_) => types::command::EXECUTION_CANCEL,
            Command::WorkspaceCreate(_) => types::command::WORKSPACE_CREATE,
            Command::WorkspaceList(_) => types::command::WORKSPACE_LIST,
            Command::AgentDefinitionCreate(_) => types::command::AGENT_DEFINITION_CREATE,
            Command::AgentDefinitionList(_) => types::command::AGENT_DEFINITION_LIST,
            Command::AgentInstanceCreate(_) => types::command::AGENT_INSTANCE_CREATE,
            Command::AgentInstanceList(_) => types::command::AGENT_INSTANCE_LIST,
            Command::AgentInstanceDelete(_) => types::command::AGENT_INSTANCE_DELETE,
            Command::ToolList(_) => types::command::TOOL_LIST,
        }
    }

    pub fn from_event(event: &CloudEvent) -> Result<Self, ProtocolError> {
        match event.type_.as_str() {
            types::command::SUBSCRIPTION_ADD => Ok(Command::SubscriptionAdd(event.data_as()?)),
            types::command::SUBSCRIPTION_REMOVE => {
                Ok(Command::SubscriptionRemove(event.data_as()?))
            }
            types::command::SUBSCRIPTION_RESUME => {
                Ok(Command::SubscriptionResume(event.data_as()?))
            }
            types::command::TASK_SUBMIT => Ok(Command::TaskSubmit(event.data_as()?)),
            types::command::EXECUTION_CANCEL => Ok(Command::ExecutionCancel(event.data_as()?)),
            types::command::WORKSPACE_CREATE => Ok(Command::WorkspaceCreate(event.data_as()?)),
            types::command::WORKSPACE_LIST => Ok(Command::WorkspaceList(event.data_as()?)),
            types::command::AGENT_DEFINITION_CREATE => {
                Ok(Command::AgentDefinitionCreate(event.data_as()?))
            }
            types::command::AGENT_DEFINITION_LIST => {
                Ok(Command::AgentDefinitionList(event.data_as()?))
            }
            types::command::AGENT_INSTANCE_CREATE => {
                Ok(Command::AgentInstanceCreate(event.data_as()?))
            }
            types::command::AGENT_INSTANCE_LIST => Ok(Command::AgentInstanceList(event.data_as()?)),
            types::command::AGENT_INSTANCE_DELETE => {
                Ok(Command::AgentInstanceDelete(event.data_as()?))
            }
            types::command::TOOL_LIST => Ok(Command::ToolList(event.data_as()?)),
            other => Err(ProtocolError::UnknownCommand(other.to_owned())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn decodes_task_submit() {
        let event = CloudEvent::new(
            "c1",
            "/office/client/x",
            types::command::TASK_SUBMIT,
            json!({
                "company": "acme",
                "project": "ecommerce",
                "agent": "backend-engineer",
                "input": "fix checkout"
            }),
        );

        let command = Command::from_event(&event).expect("decodes");
        assert_eq!(
            command,
            Command::TaskSubmit(TaskSubmit {
                company: "acme".into(),
                project: "ecommerce".into(),
                agent: "backend-engineer".into(),
                input: "fix checkout".into(),
            })
        );
    }

    #[test]
    fn decodes_agent_definition_create() {
        let event = CloudEvent::new(
            "c1",
            "/office/client/x",
            types::command::AGENT_DEFINITION_CREATE,
            json!({
                "name": "Backend Engineer",
                "role": "backend",
                "prompt": "work",
                "tools": ["shell", "editor"]
            }),
        );

        let command = Command::from_event(&event).expect("decodes");
        assert_eq!(
            command,
            Command::AgentDefinitionCreate(AgentDefinitionCreate {
                name: "Backend Engineer".into(),
                role: "backend".into(),
                prompt: "work".into(),
                tools: vec!["shell".into(), "editor".into()],
            })
        );
    }

    #[test]
    fn decodes_agent_instance_create() {
        let event = CloudEvent::new(
            "c1",
            "/office/client/x",
            types::command::AGENT_INSTANCE_CREATE,
            json!({
                "agent": "backend-engineer",
                "workspace": "acme",
                "project": "ecommerce"
            }),
        );

        let command = Command::from_event(&event).expect("decodes");
        assert_eq!(
            command,
            Command::AgentInstanceCreate(AgentInstanceCreate {
                agent: "backend-engineer".into(),
                workspace: "acme".into(),
                project: "ecommerce".into(),
            })
        );
    }

    #[test]
    fn rejects_unknown_command() {
        let event = CloudEvent::new("c1", "/office/client/x", "office.nope.v1", json!({}));
        let error = Command::from_event(&event).unwrap_err();
        assert!(matches!(error, ProtocolError::UnknownCommand(t) if t == "office.nope.v1"));
    }
}
