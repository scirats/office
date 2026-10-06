use std::path::PathBuf;
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use officed::Daemon;
use officed::adapters::{PlatformWorkspaceInspector, SqliteStore};
use protocol::{CloudEvent, types};
use serde_json::json;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::UnixStream;
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::http::HeaderValue;
use workspace::{ProvisionError, Provisioned, UserProvisioner};

fn temp_socket(name: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!("office-{}-{}.sock", name, std::process::id()));
    path
}

async fn wait_for_socket(path: &std::path::Path) {
    for _ in 0..200 {
        if path.exists() {
            return;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    panic!("socket {} was never created", path.display());
}

async fn connect(path: &std::path::Path) -> WebSocketStream<UnixStream> {
    let mut request = "ws://localhost/office".into_client_request().unwrap();
    request.headers_mut().insert(
        "Sec-WebSocket-Protocol",
        HeaderValue::from_static(types::SUBPROTOCOL),
    );

    let stream = UnixStream::connect(path).await.expect("connect");
    let (websocket, _) = tokio_tungstenite::client_async(request, stream)
        .await
        .expect("handshake");
    websocket
}

async fn send<S>(websocket: &mut WebSocketStream<S>, event: CloudEvent)
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let json = event.to_json().expect("serializes");
    websocket.send(Message::text(json)).await.expect("send");
}

async fn recv<S>(websocket: &mut WebSocketStream<S>) -> CloudEvent
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    loop {
        let message = tokio::time::timeout(Duration::from_secs(5), websocket.next())
            .await
            .expect("timed out waiting for an event")
            .expect("stream ended")
            .expect("websocket error");

        match message {
            Message::Text(text) => {
                return CloudEvent::from_json(text.as_str()).expect("valid event");
            }
            Message::Binary(bytes) => {
                let text = std::str::from_utf8(&bytes).expect("utf8");
                return CloudEvent::from_json(text).expect("valid event");
            }
            _ => continue,
        }
    }
}

fn command(type_: &str, data: serde_json::Value) -> CloudEvent {
    CloudEvent::new("cmd", "/office/client/test", type_, data)
}

struct FakeProvisioner;

impl UserProvisioner for FakeProvisioner {
    fn provision(&self, slug: &str) -> Result<Provisioned, ProvisionError> {
        Ok(Provisioned {
            user: slug.to_owned(),
            home: format!("/home/{slug}"),
        })
    }
}

#[tokio::test]
async fn creates_workspace_through_provisioner() {
    let socket = temp_socket("workspace");
    let store = SqliteStore::open_in_memory().expect("opens store");
    let daemon = Daemon::with_services(
        64,
        Box::new(FakeProvisioner),
        Box::new(store.clone()),
        Box::new(store),
        Box::new(PlatformWorkspaceInspector),
    );
    let server = tokio::spawn(daemon.serve(socket.clone(), None));
    wait_for_socket(&socket).await;

    let mut websocket = connect(&socket).await;
    send(
        &mut websocket,
        command(types::command::SUBSCRIPTION_ADD, json!({ "filter": {} })),
    )
    .await;
    assert_eq!(
        recv(&mut websocket).await.type_,
        types::event::COMMAND_ACCEPTED
    );

    send(
        &mut websocket,
        command(
            types::command::WORKSPACE_CREATE,
            json!({ "name": "Acme Corp" }),
        ),
    )
    .await;

    let mut created = None;
    for _ in 0..8 {
        let event = recv(&mut websocket).await;
        if event.type_ == types::event::WORKSPACE_CREATED {
            created = Some(event);
            break;
        }
    }

    let created = created.expect("workspace.created");
    assert_eq!(created.data["name"], "Acme Corp");
    assert_eq!(created.data["slug"], "acme-corp");
    assert_eq!(created.data["user"], "acme-corp");
    assert_eq!(created.data["home"], "/home/acme-corp");
    assert_eq!(created.subject.as_deref(), Some("workspace:acme-corp"));

    send(
        &mut websocket,
        command(types::command::WORKSPACE_LIST, json!({})),
    )
    .await;

    let listed = recv(&mut websocket).await;
    assert_eq!(listed.type_, types::event::WORKSPACE_LISTED);
    assert_eq!(listed.data["workspaces"][0]["slug"], "acme-corp");
    assert_eq!(listed.data["workspaces"][0]["home"], "/home/acme-corp");

    server.abort();
    let _ = std::fs::remove_file(&socket);
}

#[tokio::test]
async fn subscribes_submits_and_streams_execution_events() {
    let socket = temp_socket("flow");
    let store = SqliteStore::open_in_memory().expect("opens store");
    let daemon = Daemon::new(
        Box::new(store.clone()),
        Box::new(store),
        Box::new(PlatformWorkspaceInspector),
    );
    let server = tokio::spawn(daemon.serve(socket.clone(), None));
    wait_for_socket(&socket).await;

    let mut websocket = connect(&socket).await;

    send(
        &mut websocket,
        command(types::command::SUBSCRIPTION_ADD, json!({ "filter": {} })),
    )
    .await;
    let accepted = recv(&mut websocket).await;
    assert_eq!(accepted.type_, types::event::COMMAND_ACCEPTED);

    send(
        &mut websocket,
        command(
            types::command::TASK_SUBMIT,
            json!({
                "company": "acme",
                "project": "ecommerce",
                "agent": "backend-engineer",
                "input": "fix checkout"
            }),
        ),
    )
    .await;

    let mut saw_task_accepted = false;
    let mut saw_state_changed = false;
    let mut saw_log = false;
    let mut saw_succeeded = false;
    let mut execution_id = None;

    for _ in 0..64 {
        let event = recv(&mut websocket).await;
        match event.type_.as_str() {
            types::event::TASK_ACCEPTED => {
                saw_task_accepted = true;
                execution_id = event.data["execution_id"].as_str().map(str::to_owned);
            }
            types::event::EXECUTION_STATE_CHANGED => {
                saw_state_changed = true;
                if event.data["to"] == "SUCCEEDED" {
                    saw_succeeded = true;
                    break;
                }
            }
            types::event::EXECUTION_LOG => saw_log = true,
            _ => {}
        }
    }

    assert!(saw_task_accepted, "expected task.accepted");
    assert!(saw_state_changed, "expected execution.state_changed");
    assert!(saw_log, "expected execution.log");
    assert!(saw_succeeded, "expected execution to reach SUCCEEDED");

    let execution_id = execution_id.expect("task.accepted carries execution_id");
    let subject = format!("execution:{execution_id}");
    send(
        &mut websocket,
        command(
            types::command::SUBSCRIPTION_RESUME,
            json!({ "subject": subject, "last_sequence": 0 }),
        ),
    )
    .await;

    let accepted = recv(&mut websocket).await;
    assert_eq!(accepted.type_, types::event::COMMAND_ACCEPTED);

    let replayed = recv(&mut websocket).await;
    assert_eq!(replayed.type_, types::event::EXECUTION_STATE_CHANGED);
    assert_eq!(replayed.subject.as_deref(), Some(subject.as_str()));
    assert!(replayed.sequence.is_some());

    server.abort();
    let _ = std::fs::remove_file(&socket);
}

#[tokio::test]
async fn defines_links_and_rejects_duplicate_agent_instance() {
    let socket = temp_socket("agents");
    let store = SqliteStore::open_in_memory().expect("opens store");
    let daemon = Daemon::with_services(
        64,
        Box::new(FakeProvisioner),
        Box::new(store.clone()),
        Box::new(store),
        Box::new(PlatformWorkspaceInspector),
    );
    let server = tokio::spawn(daemon.serve(socket.clone(), None));
    wait_for_socket(&socket).await;

    let mut websocket = connect(&socket).await;
    send(
        &mut websocket,
        command(types::command::SUBSCRIPTION_ADD, json!({ "filter": {} })),
    )
    .await;
    assert_eq!(
        recv(&mut websocket).await.type_,
        types::event::COMMAND_ACCEPTED
    );

    send(
        &mut websocket,
        command(
            types::command::AGENT_DEFINITION_CREATE,
            json!({
                "name": "Backend Engineer",
                "role": "backend",
                "prompt": "work",
                "tools": ["shell", "editor"]
            }),
        ),
    )
    .await;
    assert_eq!(
        recv(&mut websocket).await.type_,
        types::event::COMMAND_ACCEPTED
    );
    let created = recv(&mut websocket).await;
    assert_eq!(created.type_, types::event::AGENT_DEFINITION_CREATED);
    assert_eq!(created.data["id"], "backend-engineer");
    assert_eq!(created.data["version"], 1);
    assert_eq!(created.data["tools"][0]["id"], "shell");
    assert_eq!(created.data["tools"][1]["id"], "editor");
    assert_eq!(created.subject.as_deref(), Some("agent:backend-engineer"));

    send(
        &mut websocket,
        command(types::command::AGENT_DEFINITION_LIST, json!({})),
    )
    .await;
    let listed = recv(&mut websocket).await;
    assert_eq!(listed.type_, types::event::AGENT_DEFINITION_LISTED);
    assert_eq!(listed.data["definitions"][0]["id"], "backend-engineer");

    send(
        &mut websocket,
        command(
            types::command::AGENT_INSTANCE_CREATE,
            json!({
                "agent": "backend-engineer",
                "workspace": "acme",
                "project": "ecommerce"
            }),
        ),
    )
    .await;
    assert_eq!(
        recv(&mut websocket).await.type_,
        types::event::COMMAND_ACCEPTED
    );
    let instance = recv(&mut websocket).await;
    assert_eq!(instance.type_, types::event::AGENT_INSTANCE_CREATED);
    assert_eq!(instance.data["agent"], "backend-engineer");
    assert_eq!(instance.data["workspace"], "acme");
    assert_eq!(instance.data["project"], "ecommerce");

    send(
        &mut websocket,
        command(
            types::command::AGENT_INSTANCE_CREATE,
            json!({
                "agent": "backend-engineer",
                "workspace": "acme",
                "project": "ecommerce"
            }),
        ),
    )
    .await;
    let rejected = recv(&mut websocket).await;
    assert_eq!(rejected.type_, types::event::COMMAND_REJECTED);
    assert_eq!(rejected.data["code"], "command_failed");

    server.abort();
    let _ = std::fs::remove_file(&socket);
}

#[tokio::test]
async fn lists_tools_and_rejects_unknown_tool() {
    let socket = temp_socket("tools");
    let store = SqliteStore::open_in_memory().expect("opens store");
    let daemon = Daemon::with_services(
        64,
        Box::new(FakeProvisioner),
        Box::new(store.clone()),
        Box::new(store),
        Box::new(PlatformWorkspaceInspector),
    );
    let server = tokio::spawn(daemon.serve(socket.clone(), None));
    wait_for_socket(&socket).await;

    let mut websocket = connect(&socket).await;
    send(
        &mut websocket,
        command(types::command::TOOL_LIST, json!({})),
    )
    .await;
    let listed = recv(&mut websocket).await;
    assert_eq!(listed.type_, types::event::TOOL_LISTED);
    assert!(
        listed.data["tools"]
            .as_array()
            .expect("tools array")
            .iter()
            .any(|tool| tool["id"] == "shell")
    );

    send(
        &mut websocket,
        command(
            types::command::AGENT_DEFINITION_CREATE,
            json!({
                "name": "Ghost",
                "role": "backend",
                "prompt": "p",
                "tools": ["nope"]
            }),
        ),
    )
    .await;
    let rejected = recv(&mut websocket).await;
    assert_eq!(rejected.type_, types::event::COMMAND_REJECTED);
    assert_eq!(rejected.data["code"], "command_failed");

    server.abort();
    let _ = std::fs::remove_file(&socket);
}

#[tokio::test]
async fn rejects_handshake_without_expected_subprotocol() {
    let socket = temp_socket("subprotocol");
    let store = SqliteStore::open_in_memory().expect("opens store");
    let daemon = Daemon::new(
        Box::new(store.clone()),
        Box::new(store),
        Box::new(PlatformWorkspaceInspector),
    );
    let server = tokio::spawn(daemon.serve(socket.clone(), None));
    wait_for_socket(&socket).await;

    let request = "ws://localhost/office".into_client_request().unwrap();
    let stream = UnixStream::connect(&socket).await.expect("connect");
    let result = tokio_tungstenite::client_async(request, stream).await;

    assert!(result.is_err(), "handshake must be rejected");

    server.abort();
    let _ = std::fs::remove_file(&socket);
}
