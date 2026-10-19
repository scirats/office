use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use futures_util::stream;
use futures_util::{SinkExt, StreamExt};
use inference::{ByteStream, HttpClient, HttpError};
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

use model::{Capabilities, Model, ModelCatalog, ModelId, Provider, ProviderId, ProviderKind};
use secret::{Secret, SecretStore};

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

    fn deprovision(&self, _user: &str, _home: &str) -> Result<(), ProvisionError> {
        Ok(())
    }
}

struct FakeModels;

impl ModelCatalog for FakeModels {
    fn list(&self) -> Vec<Model> {
        vec![Model {
            id: ModelId("claude-sonnet".to_owned()),
            provider: ProviderId("anthropic".to_owned()),
            api_name: "claude-sonnet-4".to_owned(),
            display_name: "Claude Sonnet".to_owned(),
            context_window: 200_000,
            max_output_tokens: 8_192,
            capabilities: Capabilities {
                tool_calling: true,
                vision: true,
                reasoning: false,
            },
        }]
    }

    fn get(&self, id: &str) -> Option<Model> {
        self.list()
            .into_iter()
            .find(|model| model.id.as_str() == id)
    }

    fn provider(&self, id: &str) -> Option<Provider> {
        (id == "anthropic").then(|| Provider {
            id: ProviderId("anthropic".to_owned()),
            kind: ProviderKind::Anthropic,
            endpoint: "https://api.anthropic.com".to_owned(),
            credential_ref: "provider/anthropic".to_owned(),
        })
    }
}

struct FakeSecrets;

impl SecretStore for FakeSecrets {
    fn resolve(&self, reference: &str) -> Option<Secret> {
        (reference == "provider/anthropic").then(|| Secret::new("sk-test"))
    }

    fn references(&self) -> Vec<String> {
        vec!["provider/anthropic".to_owned()]
    }

    fn set(&self, _reference: &str, _value: &str) -> Result<(), secret::SecretError> {
        Ok(())
    }

    fn reload(&self) -> Result<(), secret::SecretError> {
        Ok(())
    }
}

struct FakeHttp;

impl HttpClient for FakeHttp {
    fn post_stream(
        &self,
        _url: String,
        _headers: Vec<(String, String)>,
        _body: serde_json::Value,
    ) -> futures_util::future::BoxFuture<'static, Result<ByteStream, HttpError>> {
        Box::pin(async move {
            let body = "data: {\"choices\":[{\"delta\":{\"content\":\"hello \"}}]}\n\n\
                        data: {\"choices\":[{\"delta\":{\"content\":\"world\"}}]}\n\n\
                        data: [DONE]\n\n";
            let stream = stream::iter(vec![
                Ok(body.as_bytes().to_vec()) as Result<Vec<u8>, HttpError>
            ]);
            Ok(Box::pin(stream) as ByteStream)
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
        Box::new(store.clone()),
        Box::new(store),
        Box::new(PlatformWorkspaceInspector),
        Box::new(FakeModels),
        Box::new(FakeSecrets),
        Arc::new(FakeHttp),
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
    let daemon = Daemon::with_services(
        64,
        Box::new(FakeProvisioner),
        Box::new(store.clone()),
        Box::new(store.clone()),
        Box::new(store),
        Box::new(PlatformWorkspaceInspector),
        Box::new(FakeModels),
        Box::new(FakeSecrets),
        Arc::new(FakeHttp),
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
            types::command::AGENT_DEFINITION_CREATE,
            json!({
                "name": "Backend Engineer",
                "role": "backend",
                "prompt": "work",
                "tools": ["shell"],
                "model": "claude-sonnet"
            }),
        ),
    )
    .await;
    assert_eq!(
        recv(&mut websocket).await.type_,
        types::event::COMMAND_ACCEPTED
    );
    assert_eq!(
        recv(&mut websocket).await.type_,
        types::event::AGENT_DEFINITION_CREATED
    );

    send(
        &mut websocket,
        command(
            types::command::TASK_SUBMIT,
            json!({
                "company": "acme",
                "project": "",
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
        Box::new(store.clone()),
        Box::new(store),
        Box::new(PlatformWorkspaceInspector),
        Box::new(FakeModels),
        Box::new(FakeSecrets),
        Arc::new(FakeHttp),
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
        command(types::command::WORKSPACE_CREATE, json!({ "name": "Acme" })),
    )
    .await;
    assert_eq!(
        recv(&mut websocket).await.type_,
        types::event::COMMAND_ACCEPTED
    );
    assert_eq!(
        recv(&mut websocket).await.type_,
        types::event::WORKSPACE_CREATED
    );

    send(
        &mut websocket,
        command(
            types::command::PROJECT_CREATE,
            json!({ "workspace": "acme", "name": "Ecommerce" }),
        ),
    )
    .await;
    assert_eq!(
        recv(&mut websocket).await.type_,
        types::event::COMMAND_ACCEPTED
    );
    let project = recv(&mut websocket).await;
    assert_eq!(project.type_, types::event::PROJECT_CREATED);
    assert_eq!(project.data["id"], "ecommerce");

    send(
        &mut websocket,
        command(
            types::command::AGENT_DEFINITION_CREATE,
            json!({
                "name": "Backend Engineer",
                "role": "backend",
                "prompt": "work",
                "tools": ["shell", "editor"],
                "model": "claude-sonnet"
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
    assert_eq!(created.data["model"], "claude-sonnet");
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

    send(
        &mut websocket,
        command(
            types::command::AGENT_DEFINITION_DELETE,
            json!({ "id": "backend-engineer" }),
        ),
    )
    .await;
    let still_linked = recv(&mut websocket).await;
    assert_eq!(still_linked.type_, types::event::COMMAND_REJECTED);
    assert_eq!(still_linked.data["code"], "command_failed");

    send(
        &mut websocket,
        command(
            types::command::AGENT_INSTANCE_DELETE,
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
    assert_eq!(
        recv(&mut websocket).await.type_,
        types::event::AGENT_INSTANCE_DELETED
    );

    send(
        &mut websocket,
        command(
            types::command::AGENT_DEFINITION_DELETE,
            json!({ "id": "backend-engineer" }),
        ),
    )
    .await;
    assert_eq!(
        recv(&mut websocket).await.type_,
        types::event::COMMAND_ACCEPTED
    );
    let deleted = recv(&mut websocket).await;
    assert_eq!(deleted.type_, types::event::AGENT_DEFINITION_DELETED);
    assert_eq!(deleted.data["id"], "backend-engineer");

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
        Box::new(store.clone()),
        Box::new(store),
        Box::new(PlatformWorkspaceInspector),
        Box::new(FakeModels),
        Box::new(FakeSecrets),
        Arc::new(FakeHttp),
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
        command(types::command::MODEL_LIST, json!({})),
    )
    .await;
    let models = recv(&mut websocket).await;
    assert_eq!(models.type_, types::event::MODEL_LISTED);
    assert_eq!(models.data["models"][0]["id"], "claude-sonnet");
    assert_eq!(models.data["models"][0]["provider"], "anthropic");
    assert_eq!(models.data["models"][0]["provider_kind"], "anthropic");
    assert_eq!(models.data["models"][0]["credential_configured"], true);

    send(
        &mut websocket,
        command(types::command::SECRET_LIST, json!({})),
    )
    .await;
    let secrets = recv(&mut websocket).await;
    assert_eq!(secrets.type_, types::event::SECRET_LISTED);
    assert_eq!(secrets.data["references"][0], "provider/anthropic");

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

#[tokio::test]
async fn deletes_workspace_through_provisioner() {
    let socket = temp_socket("workspace-delete");
    let store = SqliteStore::open_in_memory().expect("opens store");
    let daemon = Daemon::with_services(
        64,
        Box::new(FakeProvisioner),
        Box::new(store.clone()),
        Box::new(store.clone()),
        Box::new(store),
        Box::new(PlatformWorkspaceInspector),
        Box::new(FakeModels),
        Box::new(FakeSecrets),
        Arc::new(FakeHttp),
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
    assert_eq!(
        recv(&mut websocket).await.type_,
        types::event::COMMAND_ACCEPTED
    );
    assert_eq!(
        recv(&mut websocket).await.type_,
        types::event::WORKSPACE_CREATED
    );

    send(
        &mut websocket,
        command(
            types::command::WORKSPACE_DELETE,
            json!({ "slug": "acme-corp" }),
        ),
    )
    .await;
    assert_eq!(
        recv(&mut websocket).await.type_,
        types::event::COMMAND_ACCEPTED
    );
    let deleted = recv(&mut websocket).await;
    assert_eq!(deleted.type_, types::event::WORKSPACE_DELETED);
    assert_eq!(deleted.data["slug"], "acme-corp");
    assert_eq!(deleted.subject.as_deref(), Some("workspace:acme-corp"));

    send(
        &mut websocket,
        command(types::command::WORKSPACE_LIST, json!({})),
    )
    .await;
    let listed = recv(&mut websocket).await;
    assert_eq!(listed.type_, types::event::WORKSPACE_LISTED);
    assert_eq!(
        listed.data["workspaces"].as_array().expect("array").len(),
        0
    );

    server.abort();
    let _ = std::fs::remove_file(&socket);
}

#[tokio::test]
async fn blocks_deleting_workspace_with_linked_agents() {
    let socket = temp_socket("workspace-in-use");
    let store = SqliteStore::open_in_memory().expect("opens store");
    let daemon = Daemon::with_services(
        64,
        Box::new(FakeProvisioner),
        Box::new(store.clone()),
        Box::new(store.clone()),
        Box::new(store),
        Box::new(PlatformWorkspaceInspector),
        Box::new(FakeModels),
        Box::new(FakeSecrets),
        Arc::new(FakeHttp),
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
    assert_eq!(
        recv(&mut websocket).await.type_,
        types::event::COMMAND_ACCEPTED
    );
    assert_eq!(
        recv(&mut websocket).await.type_,
        types::event::WORKSPACE_CREATED
    );

    send(
        &mut websocket,
        command(
            types::command::AGENT_DEFINITION_CREATE,
            json!({
                "name": "Backend Engineer",
                "role": "backend",
                "prompt": "work",
                "tools": ["shell"],
                "model": "claude-sonnet"
            }),
        ),
    )
    .await;
    assert_eq!(
        recv(&mut websocket).await.type_,
        types::event::COMMAND_ACCEPTED
    );
    assert_eq!(
        recv(&mut websocket).await.type_,
        types::event::AGENT_DEFINITION_CREATED
    );

    send(
        &mut websocket,
        command(
            types::command::PROJECT_CREATE,
            json!({ "workspace": "acme-corp", "name": "Ecommerce" }),
        ),
    )
    .await;
    assert_eq!(
        recv(&mut websocket).await.type_,
        types::event::COMMAND_ACCEPTED
    );
    assert_eq!(
        recv(&mut websocket).await.type_,
        types::event::PROJECT_CREATED
    );

    send(
        &mut websocket,
        command(
            types::command::AGENT_INSTANCE_CREATE,
            json!({
                "agent": "backend-engineer",
                "workspace": "acme-corp",
                "project": "ecommerce"
            }),
        ),
    )
    .await;
    assert_eq!(
        recv(&mut websocket).await.type_,
        types::event::COMMAND_ACCEPTED
    );
    assert_eq!(
        recv(&mut websocket).await.type_,
        types::event::AGENT_INSTANCE_CREATED
    );

    send(
        &mut websocket,
        command(
            types::command::WORKSPACE_DELETE,
            json!({ "slug": "acme-corp" }),
        ),
    )
    .await;
    let rejected = recv(&mut websocket).await;
    assert_eq!(rejected.type_, types::event::COMMAND_REJECTED);
    assert_eq!(rejected.data["code"], "command_failed");

    send(
        &mut websocket,
        command(types::command::WORKSPACE_LIST, json!({})),
    )
    .await;
    let listed = recv(&mut websocket).await;
    assert_eq!(listed.type_, types::event::WORKSPACE_LISTED);
    assert_eq!(listed.data["workspaces"][0]["slug"], "acme-corp");

    server.abort();
    let _ = std::fs::remove_file(&socket);
}

#[tokio::test]
async fn sets_a_secret_for_a_model_and_rejects_unknown() {
    let socket = temp_socket("secret-set");
    let store = SqliteStore::open_in_memory().expect("opens store");
    let daemon = Daemon::with_services(
        64,
        Box::new(FakeProvisioner),
        Box::new(store.clone()),
        Box::new(store.clone()),
        Box::new(store),
        Box::new(PlatformWorkspaceInspector),
        Box::new(FakeModels),
        Box::new(FakeSecrets),
        Arc::new(FakeHttp),
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
            types::command::SECRET_SET,
            json!({ "model": "claude-sonnet", "value": "sk-ant-xyz" }),
        ),
    )
    .await;
    assert_eq!(
        recv(&mut websocket).await.type_,
        types::event::COMMAND_ACCEPTED
    );
    let updated = recv(&mut websocket).await;
    assert_eq!(updated.type_, types::event::SECRET_UPDATED);
    assert_eq!(updated.data["model"], "claude-sonnet");

    send(
        &mut websocket,
        command(
            types::command::SECRET_SET,
            json!({ "model": "ghost", "value": "sk-x" }),
        ),
    )
    .await;
    assert_eq!(
        recv(&mut websocket).await.type_,
        types::event::COMMAND_REJECTED
    );

    server.abort();
    let _ = std::fs::remove_file(&socket);
}

#[tokio::test]
async fn creates_project_without_agent_and_links_multiple_agents() {
    let socket = temp_socket("projects");
    let store = SqliteStore::open_in_memory().expect("opens store");
    let daemon = Daemon::with_services(
        64,
        Box::new(FakeProvisioner),
        Box::new(store.clone()),
        Box::new(store.clone()),
        Box::new(store),
        Box::new(PlatformWorkspaceInspector),
        Box::new(FakeModels),
        Box::new(FakeSecrets),
        Arc::new(FakeHttp),
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
        command(types::command::WORKSPACE_CREATE, json!({ "name": "Acme" })),
    )
    .await;
    assert_eq!(
        recv(&mut websocket).await.type_,
        types::event::COMMAND_ACCEPTED
    );
    assert_eq!(
        recv(&mut websocket).await.type_,
        types::event::WORKSPACE_CREATED
    );

    send(
        &mut websocket,
        command(
            types::command::PROJECT_CREATE,
            json!({ "workspace": "acme", "name": "Ecommerce" }),
        ),
    )
    .await;
    assert_eq!(
        recv(&mut websocket).await.type_,
        types::event::COMMAND_ACCEPTED
    );
    let created = recv(&mut websocket).await;
    assert_eq!(created.type_, types::event::PROJECT_CREATED);
    assert_eq!(created.data["id"], "ecommerce");
    assert_eq!(created.data["workspace"], "acme");
    assert_eq!(created.subject.as_deref(), Some("project:acme/ecommerce"));

    send(
        &mut websocket,
        command(
            types::command::PROJECT_CREATE,
            json!({ "workspace": "acme", "name": "Ecommerce" }),
        ),
    )
    .await;
    assert_eq!(
        recv(&mut websocket).await.type_,
        types::event::COMMAND_REJECTED
    );

    send(
        &mut websocket,
        command(types::command::PROJECT_LIST, json!({ "workspace": "acme" })),
    )
    .await;
    let listed = recv(&mut websocket).await;
    assert_eq!(listed.type_, types::event::PROJECT_LISTED);
    assert_eq!(listed.data["projects"][0]["id"], "ecommerce");

    for (name, tools) in [
        ("Backend Engineer", json!(["shell"])),
        ("Reviewer", json!(["editor"])),
    ] {
        send(
            &mut websocket,
            command(
                types::command::AGENT_DEFINITION_CREATE,
                json!({
                    "name": name,
                    "role": "reviewer",
                    "prompt": "p",
                    "tools": tools,
                    "model": "claude-sonnet"
                }),
            ),
        )
        .await;
        assert_eq!(
            recv(&mut websocket).await.type_,
            types::event::COMMAND_ACCEPTED
        );
        assert_eq!(
            recv(&mut websocket).await.type_,
            types::event::AGENT_DEFINITION_CREATED
        );
    }

    for agent in ["backend-engineer", "reviewer"] {
        send(
            &mut websocket,
            command(
                types::command::AGENT_INSTANCE_CREATE,
                json!({ "agent": agent, "workspace": "acme", "project": "ecommerce" }),
            ),
        )
        .await;
        assert_eq!(
            recv(&mut websocket).await.type_,
            types::event::COMMAND_ACCEPTED
        );
        assert_eq!(
            recv(&mut websocket).await.type_,
            types::event::AGENT_INSTANCE_CREATED
        );
    }

    send(
        &mut websocket,
        command(
            types::command::AGENT_INSTANCE_LIST,
            json!({ "workspace": "acme", "project": "ecommerce" }),
        ),
    )
    .await;
    let instances = recv(&mut websocket).await;
    assert_eq!(instances.type_, types::event::AGENT_INSTANCE_LISTED);
    assert_eq!(
        instances.data["instances"].as_array().expect("array").len(),
        2
    );

    send(
        &mut websocket,
        command(
            types::command::AGENT_INSTANCE_CREATE,
            json!({ "agent": "reviewer", "workspace": "acme", "project": "ghost" }),
        ),
    )
    .await;
    assert_eq!(
        recv(&mut websocket).await.type_,
        types::event::COMMAND_REJECTED
    );

    server.abort();
    let _ = std::fs::remove_file(&socket);
}
