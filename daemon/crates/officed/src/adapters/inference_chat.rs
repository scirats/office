use futures_util::StreamExt;
use futures_util::future::BoxFuture;
use inference::{
    ChatEvent as WireEvent, ChatMessage as WireMessage, ChatRequest as WireRequest, ChatService,
    ChatTool as WireTool,
};
use runner::{Chat, ChatError, ChatEvent, ChatMessage, ChatStream, ChatTool};

pub struct InferenceChat {
    service: ChatService,
    endpoint: String,
    api_key: String,
    model: String,
}

impl InferenceChat {
    pub fn new(service: ChatService, endpoint: String, api_key: String, model: String) -> Self {
        Self {
            service,
            endpoint,
            api_key,
            model,
        }
    }
}

impl Chat for InferenceChat {
    fn stream(
        &self,
        messages: Vec<ChatMessage>,
        tools: Vec<ChatTool>,
    ) -> BoxFuture<'static, Result<ChatStream, ChatError>> {
        let service = self.service.clone();
        let request = WireRequest {
            endpoint: self.endpoint.clone(),
            api_key: self.api_key.clone(),
            model: self.model.clone(),
            messages: messages.into_iter().map(to_wire_message).collect(),
            tools: tools.into_iter().map(to_wire_tool).collect(),
        };

        Box::pin(async move {
            let stream = service.stream(request).await.map_err(fail)?;
            let mapped = stream.map(|item| item.map(from_wire_event).map_err(fail));
            Ok(Box::pin(mapped) as ChatStream)
        })
    }
}

fn fail(error: inference::ChatError) -> ChatError {
    ChatError::Failed(error.to_string())
}

fn to_wire_message(message: ChatMessage) -> WireMessage {
    match message {
        ChatMessage::System { content } => WireMessage::System { content },
        ChatMessage::User { content } => WireMessage::User { content },
        ChatMessage::Assistant {
            content,
            tool_calls,
        } => WireMessage::Assistant {
            content,
            tool_calls: tool_calls.into_iter().map(to_wire_call).collect(),
        },
        ChatMessage::Tool {
            tool_call_id,
            name,
            content,
        } => WireMessage::Tool {
            tool_call_id,
            name,
            content,
        },
    }
}

fn to_wire_call(call: runner::ToolCall) -> inference::ToolCall {
    inference::ToolCall {
        id: call.id,
        name: call.name,
        arguments: call.arguments,
    }
}

fn to_wire_tool(tool: ChatTool) -> WireTool {
    WireTool {
        name: tool.name,
        description: tool.description,
        parameters: tool.parameters,
    }
}

fn from_wire_event(event: WireEvent) -> ChatEvent {
    match event {
        WireEvent::Text(text) => ChatEvent::Text(text),
        WireEvent::ToolCall(call) => ChatEvent::ToolCall(runner::ToolCall {
            id: call.id,
            name: call.name,
            arguments: call.arguments,
        }),
    }
}
