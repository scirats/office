use std::sync::Arc;

use futures_util::StreamExt;

use crate::domain::{ChatEvent, ChatMessage, RunError, RunEvent, RunRequest};
use crate::ports::{Chat, ToolRunner};

pub struct RunnerService {
    chat: Arc<dyn Chat>,
    tools: Arc<dyn ToolRunner>,
}

impl RunnerService {
    pub fn new(chat: Arc<dyn Chat>, tools: Arc<dyn ToolRunner>) -> Self {
        Self { chat, tools }
    }

    pub async fn run(
        &self,
        request: RunRequest,
        emit: &mut (dyn FnMut(RunEvent) + Send),
    ) -> Result<(), RunError> {
        let mut messages = vec![
            ChatMessage::System {
                content: request.system.clone(),
            },
            ChatMessage::User {
                content: request.input.clone(),
            },
        ];

        for _ in 0..request.max_steps {
            let mut stream = self
                .chat
                .stream(messages.clone(), request.tools.clone())
                .await?;
            let mut text = String::new();
            let mut calls = Vec::new();
            while let Some(item) = stream.next().await {
                match item? {
                    ChatEvent::Text(delta) => {
                        text.push_str(&delta);
                        emit(RunEvent::Text(delta));
                    }
                    ChatEvent::ToolCall(call) => calls.push(call),
                }
            }

            if calls.is_empty() {
                return Ok(());
            }

            messages.push(ChatMessage::Assistant {
                content: text,
                tool_calls: calls.clone(),
            });

            for call in calls {
                emit(RunEvent::ToolCall(call.clone()));
                let output = match self.tools.run(call.clone(), request.context.clone()).await {
                    Ok(output) => output,
                    Err(error) => format!("error: {error}"),
                };
                emit(RunEvent::ToolResult {
                    id: call.id.clone(),
                    name: call.name.clone(),
                    output: output.clone(),
                });
                messages.push(ChatMessage::Tool {
                    tool_call_id: call.id,
                    name: call.name,
                    content: output,
                });
            }
        }

        Err(RunError::MaxSteps(request.max_steps))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;
    use std::sync::Mutex;

    use futures_util::future::BoxFuture;
    use futures_util::stream;

    use super::*;
    use crate::domain::{ChatError, ChatTool, RunContext, RunScope, ToolCall};
    use crate::ports::{ChatStream, ToolError};

    struct ScriptedChat {
        turns: Mutex<VecDeque<Vec<Result<ChatEvent, ChatError>>>>,
    }

    impl ScriptedChat {
        fn new(turns: Vec<Vec<Result<ChatEvent, ChatError>>>) -> Self {
            Self {
                turns: Mutex::new(turns.into()),
            }
        }
    }

    impl Chat for ScriptedChat {
        fn stream(
            &self,
            _messages: Vec<ChatMessage>,
            _tools: Vec<ChatTool>,
        ) -> BoxFuture<'static, Result<ChatStream, ChatError>> {
            let turn = self.turns.lock().unwrap().pop_front().unwrap_or_default();
            Box::pin(async move {
                let stream = stream::iter(turn);
                Ok(Box::pin(stream) as ChatStream)
            })
        }
    }

    struct EchoTools;

    impl ToolRunner for EchoTools {
        fn run(
            &self,
            call: ToolCall,
            _context: RunContext,
        ) -> BoxFuture<'static, Result<String, ToolError>> {
            Box::pin(async move { Ok(format!("{} -> {}", call.name, call.arguments)) })
        }
    }

    fn call(id: &str, name: &str, arguments: &str) -> ToolCall {
        ToolCall {
            id: id.to_owned(),
            name: name.to_owned(),
            arguments: arguments.to_owned(),
        }
    }

    fn request() -> RunRequest {
        RunRequest {
            system: "be brief".to_owned(),
            input: "list files".to_owned(),
            tools: Vec::new(),
            context: RunContext {
                execution_id: "exec_1".to_owned(),
                scope: RunScope::Agent,
            },
            max_steps: 4,
        }
    }

    async fn run(turns: Vec<Vec<Result<ChatEvent, ChatError>>>) -> Vec<RunEvent> {
        let chat = Arc::new(ScriptedChat::new(turns));
        let tools = Arc::new(EchoTools);
        let service = RunnerService::new(chat, tools);
        let mut events = Vec::new();
        service
            .run(request(), &mut |event| events.push(event))
            .await
            .expect("run");
        events
    }

    #[tokio::test]
    async fn stops_when_the_model_does_not_use_tools() {
        let events = run(vec![vec![Ok(ChatEvent::Text("done".to_owned()))]]).await;
        assert_eq!(events, vec![RunEvent::Text("done".to_owned())]);
    }

    #[tokio::test]
    async fn executes_tool_calls_and_feeds_results_back() {
        let tool_turn = vec![
            Ok(ChatEvent::Text("let me check".to_owned())),
            Ok(ChatEvent::ToolCall(call(
                "c1",
                "shell",
                "{\"command\":\"ls\"}",
            ))),
        ];
        let final_turn = vec![Ok(ChatEvent::Text("all good".to_owned()))];
        let events = run(vec![tool_turn, final_turn]).await;

        assert_eq!(
            events,
            vec![
                RunEvent::Text("let me check".to_owned()),
                RunEvent::ToolCall(call("c1", "shell", "{\"command\":\"ls\"}")),
                RunEvent::ToolResult {
                    id: "c1".to_owned(),
                    name: "shell".to_owned(),
                    output: "shell -> {\"command\":\"ls\"}".to_owned(),
                },
                RunEvent::Text("all good".to_owned()),
            ]
        );
    }

    #[tokio::test]
    async fn fails_after_the_step_budget() {
        let turns: Vec<Vec<Result<ChatEvent, ChatError>>> = (0..4)
            .map(|_| vec![Ok(ChatEvent::ToolCall(call("c", "shell", "{}")))])
            .collect();
        let chat = Arc::new(ScriptedChat::new(turns));
        let service = RunnerService::new(chat, Arc::new(EchoTools));
        let error = service
            .run(request(), &mut |_| {})
            .await
            .expect_err("budget");
        assert!(matches!(error, RunError::MaxSteps(4)));
    }
}
