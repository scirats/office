use futures_util::future::BoxFuture;
use futures_util::stream::BoxStream;

use crate::domain::{ChatError, ChatEvent, ChatMessage, ChatTool, RunContext, ToolCall};

pub type ChatStream = BoxStream<'static, Result<ChatEvent, ChatError>>;

pub trait Chat: Send + Sync {
    fn stream(
        &self,
        messages: Vec<ChatMessage>,
        tools: Vec<ChatTool>,
    ) -> BoxFuture<'static, Result<ChatStream, ChatError>>;
}

#[derive(Debug, thiserror::Error)]
pub enum ToolError {
    #[error("tool '{name}' failed: {message}")]
    Failed { name: String, message: String },
}

pub trait ToolRunner: Send + Sync {
    fn run(
        &self,
        call: ToolCall,
        context: RunContext,
    ) -> BoxFuture<'static, Result<String, ToolError>>;
}
