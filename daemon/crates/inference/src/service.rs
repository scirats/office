use std::collections::VecDeque;
use std::sync::Arc;

use futures_util::future::BoxFuture;
use futures_util::stream::{BoxStream, StreamExt, unfold};
use serde_json::Value;

use crate::domain::{ChatError, ChatEvent, ChatMessage, ChatRequest, ChatTool, ToolCall};
use crate::ports::{ByteStream, HttpClient};

pub type ChatStream = BoxStream<'static, Result<ChatEvent, ChatError>>;

#[derive(Clone)]
pub struct ChatService {
    client: Arc<dyn HttpClient>,
}

impl ChatService {
    pub fn new(client: Arc<dyn HttpClient>) -> Self {
        Self { client }
    }

    pub fn stream(
        &self,
        request: ChatRequest,
    ) -> BoxFuture<'static, Result<ChatStream, ChatError>> {
        let client = self.client.clone();
        Box::pin(async move {
            let url = format!(
                "{}/chat/completions",
                request.endpoint.trim_end_matches('/')
            );
            let messages: Vec<Value> = request.messages.iter().map(message_json).collect();
            let mut body = serde_json::json!({
                "model": request.model,
                "stream": true,
                "messages": messages,
            });
            if !request.tools.is_empty() {
                body["tools"] = Value::Array(request.tools.iter().map(tool_json).collect());
            }

            let headers = vec![
                (
                    "Authorization".to_owned(),
                    format!("Bearer {}", request.api_key),
                ),
                ("Content-Type".to_owned(), "application/json".to_owned()),
            ];

            let bytes = client
                .post_stream(url, headers, body)
                .await
                .map_err(|error| ChatError::Transport(error.to_string()))?;
            Ok(parse_sse(bytes))
        })
    }
}

fn message_json(message: &ChatMessage) -> Value {
    match message {
        ChatMessage::System { content } => {
            serde_json::json!({ "role": "system", "content": content })
        }
        ChatMessage::User { content } => {
            serde_json::json!({ "role": "user", "content": content })
        }
        ChatMessage::Assistant {
            content,
            tool_calls,
        } => {
            if tool_calls.is_empty() {
                serde_json::json!({ "role": "assistant", "content": content })
            } else {
                let calls: Vec<Value> = tool_calls.iter().map(tool_call_json).collect();
                serde_json::json!({
                    "role": "assistant",
                    "content": content,
                    "tool_calls": calls,
                })
            }
        }
        ChatMessage::Tool {
            tool_call_id,
            name,
            content,
        } => serde_json::json!({
            "role": "tool",
            "tool_call_id": tool_call_id,
            "name": name,
            "content": content,
        }),
    }
}

fn tool_call_json(call: &ToolCall) -> Value {
    serde_json::json!({
        "id": call.id,
        "type": "function",
        "function": { "name": call.name, "arguments": call.arguments },
    })
}

fn tool_json(tool: &ChatTool) -> Value {
    serde_json::json!({
        "type": "function",
        "function": {
            "name": tool.name,
            "description": tool.description,
            "parameters": tool.parameters,
        },
    })
}

#[derive(Default)]
struct PartialToolCall {
    index: u64,
    id: String,
    name: String,
    arguments: String,
}

struct SseState {
    bytes: ByteStream,
    buffer: String,
    pending: VecDeque<ChatEvent>,
    tool_calls: Vec<PartialToolCall>,
    failure: Option<ChatError>,
    flushed: bool,
    done: bool,
}

fn parse_sse(bytes: ByteStream) -> ChatStream {
    let initial = SseState {
        bytes,
        buffer: String::new(),
        pending: VecDeque::new(),
        tool_calls: Vec::new(),
        failure: None,
        flushed: false,
        done: false,
    };

    Box::pin(unfold(initial, |mut state| async move {
        loop {
            if let Some(event) = state.pending.pop_front() {
                return Some((Ok(event), state));
            }
            if let Some(failure) = state.failure.take() {
                return Some((Err(failure), state));
            }
            if state.done {
                return None;
            }

            match state.bytes.next().await {
                None => return None,
                Some(Err(error)) => {
                    state.done = true;
                    return Some((Err(ChatError::Transport(error.to_string())), state));
                }
                Some(Ok(chunk)) => {
                    state.buffer.push_str(&String::from_utf8_lossy(&chunk));
                    while let Some(index) = state.buffer.find('\n') {
                        let line = state.buffer[..index].trim_end_matches('\r').to_owned();
                        state.buffer.drain(..=index);
                        let Some(payload) = line.strip_prefix("data:") else {
                            continue;
                        };
                        let payload = payload.trim();
                        if payload == "[DONE]" {
                            state.flush_tool_calls();
                            state.done = true;
                            break;
                        }
                        apply_payload(&mut state, payload);
                    }
                }
            }
        }
    }))
}

impl SseState {
    fn flush_tool_calls(&mut self) {
        if self.flushed {
            return;
        }
        self.flushed = true;
        for call in self.tool_calls.drain(..) {
            self.pending.push_back(ChatEvent::ToolCall(ToolCall {
                id: call.id,
                name: call.name,
                arguments: call.arguments,
            }));
        }
    }

    fn tool_entry(&mut self, index: u64) -> &mut PartialToolCall {
        if let Some(position) = self.tool_calls.iter().position(|call| call.index == index) {
            return &mut self.tool_calls[position];
        }
        self.tool_calls.push(PartialToolCall {
            index,
            ..PartialToolCall::default()
        });
        let last = self.tool_calls.len() - 1;
        &mut self.tool_calls[last]
    }
}

fn apply_payload(state: &mut SseState, payload: &str) {
    let Ok(value) = serde_json::from_str::<Value>(payload) else {
        return;
    };

    if let Some(error) = value.get("error") {
        let message = error
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("provider error")
            .to_owned();
        state.failure = Some(ChatError::Transport(message));
        state.done = true;
        return;
    }

    let Some(choice) = value.get("choices").and_then(|choices| choices.get(0)) else {
        return;
    };

    if let Some(delta) = choice.get("delta") {
        if let Some(text) = delta.get("content").and_then(Value::as_str)
            && !text.is_empty()
        {
            state.pending.push_back(ChatEvent::Text(text.to_owned()));
        }
        if let Some(calls) = delta.get("tool_calls").and_then(Value::as_array) {
            for call in calls {
                let index = call.get("index").and_then(Value::as_u64).unwrap_or(0);
                let entry = state.tool_entry(index);
                if let Some(id) = call.get("id").and_then(Value::as_str) {
                    entry.id = id.to_owned();
                }
                if let Some(function) = call.get("function") {
                    if let Some(name) = function.get("name").and_then(Value::as_str)
                        && !name.is_empty()
                    {
                        entry.name = name.to_owned();
                    }
                    if let Some(arguments) = function.get("arguments").and_then(Value::as_str) {
                        entry.arguments.push_str(arguments);
                    }
                }
            }
        }
    }

    if let Some(reason) = choice.get("finish_reason")
        && !reason.is_null()
    {
        state.flush_tool_calls();
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use futures_util::stream;

    use super::*;
    use crate::ports::HttpError;

    #[derive(Default)]
    struct FakeHttp {
        bodies: Mutex<Vec<Vec<u8>>>,
        requests: Mutex<Vec<Value>>,
    }

    impl HttpClient for FakeHttp {
        fn post_stream(
            &self,
            _url: String,
            _headers: Vec<(String, String)>,
            body: serde_json::Value,
        ) -> BoxFuture<'static, Result<ByteStream, HttpError>> {
            let chunks = self.bodies.lock().unwrap().clone();
            self.requests.lock().unwrap().push(body);
            Box::pin(async move {
                let stream = stream::iter(
                    chunks
                        .into_iter()
                        .map(Ok::<Vec<u8>, HttpError>)
                        .collect::<Vec<_>>(),
                );
                Ok(Box::pin(stream) as ByteStream)
            })
        }
    }

    fn request() -> ChatRequest {
        ChatRequest {
            endpoint: "https://api.example.com/".to_owned(),
            api_key: "sk-test".to_owned(),
            model: "deepseek-v4-pro".to_owned(),
            messages: vec![
                ChatMessage::System {
                    content: "be brief".to_owned(),
                },
                ChatMessage::User {
                    content: "hello".to_owned(),
                },
            ],
            tools: vec![ChatTool {
                name: "shell".to_owned(),
                description: "run a command".to_owned(),
                parameters: serde_json::json!({ "type": "object" }),
            }],
        }
    }

    async fn collect(chunks: Vec<&str>) -> Vec<Result<ChatEvent, ChatError>> {
        let http = FakeHttp {
            bodies: Mutex::new(chunks.into_iter().map(|c| c.as_bytes().to_vec()).collect()),
            requests: Mutex::new(Vec::new()),
        };
        let service = ChatService::new(Arc::new(http));
        let mut stream = service.stream(request()).await.expect("starts");
        let mut out = Vec::new();
        while let Some(item) = stream.next().await {
            out.push(item);
        }
        out
    }

    #[tokio::test]
    async fn yields_text_deltas_and_stops_at_done() {
        let chunks = vec![
            "data: {\"choices\":[{\"delta\":{\"content\":\"Hel\"}}]}\n\n",
            "data: {\"choices\":[{\"delta\":{\"content\":\"lo\"}}]}\n\n",
            "data: [DONE]\n\n",
            "data: {\"choices\":[{\"delta\":{\"content\":\"ignored\"}}]}\n\n",
        ];
        let events = collect(chunks).await;
        let texts: Vec<String> = events
            .into_iter()
            .map(|item| match item.expect("event") {
                ChatEvent::Text(text) => text,
                other => panic!("unexpected {other:?}"),
            })
            .collect();
        assert_eq!(texts, vec!["Hel", "lo"]);
    }

    #[tokio::test]
    async fn reassembles_lines_split_across_chunks() {
        let chunks = vec![
            "data: {\"choices\":[{\"delta\":{\"content\":\"ab",
            "c\"}}]}\n\ndata: [DONE]\n\n",
        ];
        let events = collect(chunks).await;
        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0].as_ref().expect("event"),
            &ChatEvent::Text("abc".to_owned())
        );
    }

    #[tokio::test]
    async fn accumulates_fragmented_tool_calls() {
        let chunks = vec![
            "data: {\"choices\":[{\"delta\":{\"tool_calls\":[{\"index\":0,\"id\":\"call_1\",\
             \"function\":{\"name\":\"shell\",\"arguments\":\"{\\\"comm\"}}]}}]}\n\n",
            "data: {\"choices\":[{\"delta\":{\"tool_calls\":[{\"index\":0,\
             \"function\":{\"arguments\":\"and\\\":\\\"ls\\\"}\"}}]}}]}\n\n",
            "data: {\"choices\":[{\"finish_reason\":\"tool_calls\"}]}\n\n",
            "data: [DONE]\n\n",
        ];
        let events = collect(chunks).await;
        assert_eq!(
            events[0].as_ref().expect("event"),
            &ChatEvent::ToolCall(ToolCall {
                id: "call_1".to_owned(),
                name: "shell".to_owned(),
                arguments: "{\"command\":\"ls\"}".to_owned(),
            })
        );
    }

    #[tokio::test]
    async fn surfaces_provider_errors() {
        let chunks = vec![
            "data: {\"choices\":[{\"delta\":{\"content\":\"partial\"}}]}\n\n",
            "data: {\"error\":{\"message\":\"rate limited\"}}\n\n",
        ];
        let events = collect(chunks).await;
        assert_eq!(
            events[0].as_ref().expect("event"),
            &ChatEvent::Text("partial".to_owned())
        );
        assert!(matches!(
            &events[1],
            Err(ChatError::Transport(message)) if message == "rate limited"
        ));
    }

    #[tokio::test]
    async fn sends_messages_and_tools_in_the_body() {
        let http = Arc::new(FakeHttp {
            bodies: Mutex::new(vec![b"data: [DONE]\n\n".to_vec()]),
            requests: Mutex::new(Vec::new()),
        });
        let service = ChatService::new(http.clone());
        let mut stream = service.stream(request()).await.expect("starts");
        while stream.next().await.is_some() {}

        let requests = http.requests.lock().unwrap();
        let body = &requests[0];
        assert_eq!(body["messages"][0]["role"], "system");
        assert_eq!(body["messages"][1]["content"], "hello");
        assert_eq!(body["tools"][0]["type"], "function");
        assert_eq!(body["tools"][0]["function"]["name"], "shell");
    }
}
