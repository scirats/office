use serde_json::Value;

#[derive(Debug, Clone, PartialEq)]
pub struct ChatTool {
    pub name: String,
    pub description: String,
    pub parameters: Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ChatMessage {
    System {
        content: String,
    },
    User {
        content: String,
    },
    Assistant {
        content: String,
        tool_calls: Vec<ToolCall>,
    },
    Tool {
        tool_call_id: String,
        name: String,
        content: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum ChatEvent {
    Text(String),
    ToolCall(ToolCall),
}

#[derive(Debug, thiserror::Error)]
pub enum ChatError {
    #[error("model call failed: {0}")]
    Failed(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunScope {
    Agent,

    Instance {
        workspace: String,
        home: String,
        project: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunContext {
    pub execution_id: String,
    pub scope: RunScope,
}

impl RunContext {
    pub fn agent(execution_id: impl Into<String>) -> Self {
        Self {
            execution_id: execution_id.into(),
            scope: RunScope::Agent,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunRequest {
    pub system: String,
    pub input: String,
    pub tools: Vec<ChatTool>,
    pub context: RunContext,
    pub max_steps: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RunEvent {
    Text(String),
    ToolCall(ToolCall),
    ToolResult {
        id: String,
        name: String,
        output: String,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum RunError {
    #[error(transparent)]
    Chat(#[from] ChatError),
    #[error("model did not finish within {0} turns")]
    MaxSteps(u32),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_error_displays_chat_cause() {
        let error = RunError::from(ChatError::Failed("boom".to_owned()));
        assert_eq!(error.to_string(), "model call failed: boom");
    }
}
