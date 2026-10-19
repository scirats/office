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

#[derive(Debug, Clone, PartialEq)]
pub struct ChatRequest {
    pub endpoint: String,

    pub api_key: String,

    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub tools: Vec<ChatTool>,
}

#[derive(Debug, thiserror::Error)]
pub enum ChatError {
    #[error("provider request failed: {0}")]
    Transport(String),
}
