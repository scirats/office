use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use agent::{
    AgentDefinition, AgentInstance, AgentInstanceRecord, AgentRepository, AgentService, DefineAgent,
};
use broker::{Broker, ConnectionId, Filter, ReplayError};
use execution::{DomainEvent, ExecutionId, ExecutionService, ExecutionState, LogStream};
use futures_util::{SinkExt, StreamExt};
use inference::{ChatService, HttpClient};
use model::ModelCatalog;
use project::{Project, ProjectRecord, ProjectRepository, ProjectService};
use protocol::command::{
    AgentDefinitionCreate, AgentDefinitionDelete, AgentInstanceCreate, AgentInstanceDelete,
    AgentInstanceList, ProjectCreate, ProjectDelete, ProjectList, SecretSet, TaskSubmit,
};
use protocol::event;
use protocol::types;
use protocol::{CloudEvent, Command};
use reconciliation::{ExpectedWorkspace, ReconciliationService, WorkspaceInspector};
use runner::{ChatTool, RunContext, RunEvent, RunRequest, RunScope, RunnerService, ToolRunner};
use secret::SecretStore;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::{TcpListener, UnixListener};
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::handshake::server::{ErrorResponse, Request, Response};
use tokio_tungstenite::tungstenite::http::{HeaderValue, StatusCode};
use tool::{Tool, ToolService};
use workspace::{UserProvisioner, WorkspaceRepository, WorkspaceService};

use crate::adapters::{
    BwrapSandbox, FileModelCatalog, FileSecretStore, InferenceChat, PlatformUserProvisioner,
    ReqwestHttpClient, SystemClock, ToolExecutor, UuidV7Generator, new_connection_id,
};
use crate::bridges::Encoder;
use crate::policy::ExecutionPolicy;
use toolbox::BuiltinToolCatalog;

mod commands;
mod runs;

const DEFAULT_BROKER_CAPACITY: usize = 1024;
const DEFAULT_RECONCILE_SECS: u64 = 30;

#[derive(Clone)]
pub struct Daemon {
    inner: Arc<Mutex<Inner>>,
    workspaces: Arc<WorkspaceService>,
    projects: Arc<ProjectService>,
    agents: Arc<AgentService>,
    tools: Arc<ToolService>,
    models: Arc<dyn ModelCatalog>,
    secrets: Arc<dyn SecretStore>,
    chat: ChatService,
    tool_runner: Arc<dyn ToolRunner>,
    policy: ExecutionPolicy,
    reconciler: Arc<ReconciliationService>,
    reconcile_interval: Duration,
    encoder: Encoder,
}

struct Inner {
    executions: ExecutionService<SystemClock, UuidV7Generator>,
    broker: Broker,
    senders: HashMap<String, mpsc::UnboundedSender<Message>>,
    next_connection: u64,
}

impl Daemon {
    pub fn new(
        workspaces: Box<dyn WorkspaceRepository>,
        agents: Box<dyn AgentRepository>,
        projects: Box<dyn ProjectRepository>,
        inspector: Box<dyn WorkspaceInspector>,
    ) -> Self {
        Self::with_services(
            DEFAULT_BROKER_CAPACITY,
            Box::new(PlatformUserProvisioner),
            workspaces,
            agents,
            projects,
            inspector,
            Box::new(FileModelCatalog::empty()),
            Box::new(FileSecretStore::empty()),
            Arc::new(ReqwestHttpClient::new()),
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn with_adapters(
        provisioner: Box<dyn UserProvisioner>,
        workspaces: Box<dyn WorkspaceRepository>,
        agents: Box<dyn AgentRepository>,
        projects: Box<dyn ProjectRepository>,
        inspector: Box<dyn WorkspaceInspector>,
        models: Box<dyn ModelCatalog>,
        secrets: Box<dyn SecretStore>,
        http: Arc<dyn HttpClient>,
    ) -> Self {
        Self::with_services(
            DEFAULT_BROKER_CAPACITY,
            provisioner,
            workspaces,
            agents,
            projects,
            inspector,
            models,
            secrets,
            http,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn with_services(
        capacity: usize,
        provisioner: Box<dyn UserProvisioner>,
        workspaces: Box<dyn WorkspaceRepository>,
        agents: Box<dyn AgentRepository>,
        projects: Box<dyn ProjectRepository>,
        inspector: Box<dyn WorkspaceInspector>,
        models: Box<dyn ModelCatalog>,
        secrets: Box<dyn SecretStore>,
        http: Arc<dyn HttpClient>,
    ) -> Self {
        let policy = ExecutionPolicy::from_env();
        Self {
            inner: Arc::new(Mutex::new(Inner {
                executions: ExecutionService::new(SystemClock, UuidV7Generator),
                broker: Broker::new(capacity),
                senders: HashMap::new(),
                next_connection: 1,
            })),
            workspaces: Arc::new(WorkspaceService::new(provisioner, workspaces)),
            projects: Arc::new(ProjectService::new(projects)),
            agents: Arc::new(AgentService::new(agents, Box::new(UuidV7Generator))),
            tools: Arc::new(ToolService::new(Box::new(BuiltinToolCatalog))),
            models: Arc::from(models),
            secrets: Arc::from(secrets),
            chat: ChatService::new(http),
            tool_runner: Arc::new(ToolExecutor::new(
                Arc::new(BwrapSandbox::new(policy.cgroup_root.clone())),
                policy.clone(),
            )),
            policy,
            reconciler: Arc::new(ReconciliationService::new(inspector)),
            reconcile_interval: Duration::from_secs(DEFAULT_RECONCILE_SECS),
            encoder: Encoder,
        }
    }

    pub fn with_reconcile_interval(mut self, interval: Duration) -> Self {
        self.reconcile_interval = interval;
        self
    }

    pub fn with_tool_runner(mut self, runner: Arc<dyn ToolRunner>) -> Self {
        self.tool_runner = runner;
        self
    }

    pub fn reconcile(&self) {
        self.reload_secrets();
        let records = match self.workspaces.list() {
            Ok(records) => records,
            Err(error) => {
                tracing::error!(
                    target: "office.reconciliation",
                    error = %error,
                    "could not read workspaces"
                );
                return;
            }
        };

        let previous: HashMap<String, String> = records
            .iter()
            .map(|record| (record.slug.clone(), record.status.clone()))
            .collect();

        let expected: Vec<ExpectedWorkspace> = records
            .into_iter()
            .map(|record| ExpectedWorkspace {
                slug: record.slug,
                user: record.user,
                home: record.home,
            })
            .collect();

        let report = offload(|| self.reconciler.run(&expected));

        for reconciliation in &report.workspaces {
            let status = reconciliation.status.as_str();
            if previous.get(&reconciliation.slug).map(String::as_str) == Some(status) {
                continue;
            }

            if let Err(error) = self.workspaces.set_status(&reconciliation.slug, status) {
                tracing::error!(
                    target: "office.reconciliation",
                    workspace = %reconciliation.slug,
                    error = %error,
                    "could not persist status"
                );
            }

            let subject = format!("workspace:{}", reconciliation.slug);
            let payload = event::WorkspaceStatusChanged {
                slug: reconciliation.slug.clone(),
                status: status.to_owned(),
                inconsistencies: reconciliation
                    .inconsistencies
                    .iter()
                    .map(|item| item.as_str().to_owned())
                    .collect(),
            };

            if payload.inconsistencies.is_empty() {
                tracing::info!(
                    target: "office.reconciliation",
                    workspace = %reconciliation.slug,
                    "workspace active"
                );
            } else {
                tracing::warn!(
                    target: "office.security",
                    workspace = %reconciliation.slug,
                    status,
                    inconsistencies = ?payload.inconsistencies,
                    "workspace inconsistent"
                );
            }

            let status_event = self.encoder.workspace_status_changed(&subject, payload);
            let mut inner = self.state();
            publish(&mut inner, &subject, status_event);
        }

        self.reconcile_domain();
    }

    fn reconcile_domain(&self) {
        use std::collections::HashSet;

        let workspaces: HashSet<String> = match self.workspaces.list() {
            Ok(records) => records.into_iter().map(|record| record.slug).collect(),
            Err(error) => {
                tracing::error!(
                    target: "office.reconciliation",
                    error = %error,
                    "could not read workspaces for domain reconciliation"
                );
                return;
            }
        };

        match self.projects.list(None) {
            Ok(projects) => {
                for project in projects {
                    if !workspaces.contains(&project.workspace) {
                        tracing::warn!(
                            target: "office.reconciliation",
                            workspace = %project.workspace,
                            project = %project.id,
                            "project references a missing workspace"
                        );
                    }
                }
            }
            Err(error) => tracing::error!(
                target: "office.reconciliation",
                error = %error,
                "could not read projects"
            ),
        }

        match self.agents.list_instances(None, None) {
            Ok(instances) => {
                for instance in instances {
                    if !workspaces.contains(&instance.workspace) {
                        tracing::warn!(
                            target: "office.reconciliation",
                            workspace = %instance.workspace,
                            instance = %instance.id,
                            "agent instance references a missing workspace"
                        );
                    }
                }
            }
            Err(error) => tracing::error!(
                target: "office.reconciliation",
                error = %error,
                "could not read agent instances"
            ),
        }
    }

    async fn reconcile_loop(self, interval: Duration) {
        if interval.is_zero() {
            return;
        }
        let mut ticker = tokio::time::interval_at(tokio::time::Instant::now() + interval, interval);
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        loop {
            ticker.tick().await;
            self.reconcile();
        }
    }

    pub async fn serve(
        self,
        socket_path: impl AsRef<Path>,
        tcp_addr: Option<SocketAddr>,
    ) -> std::io::Result<()> {
        let unix = bind_unix(socket_path.as_ref())?;

        let interval = self.reconcile_interval;
        let reconcile_daemon = self.clone();
        tokio::spawn(async move { reconcile_daemon.reconcile_loop(interval).await });

        match tcp_addr {
            Some(addr) => {
                let tcp = TcpListener::bind(addr).await?;
                let unix_daemon = self.clone();
                let tcp_daemon = self.clone();
                tokio::try_join!(
                    async move { unix_daemon.accept_unix(unix).await },
                    async move { tcp_daemon.accept_tcp(tcp).await },
                )?;
            }
            None => self.accept_unix(unix).await?,
        }

        Ok(())
    }

    async fn accept_unix(self, listener: UnixListener) -> std::io::Result<()> {
        loop {
            match listener.accept().await {
                Ok((stream, _)) => self.spawn_connection(stream),
                Err(error) => tracing::warn!(error = %error, "unix accept failed"),
            }
        }
    }

    async fn accept_tcp(self, listener: TcpListener) -> std::io::Result<()> {
        loop {
            match listener.accept().await {
                Ok((stream, _)) => self.spawn_connection(stream),
                Err(error) => tracing::warn!(error = %error, "tcp accept failed"),
            }
        }
    }

    fn spawn_connection<S>(&self, stream: S)
    where
        S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
    {
        let daemon = self.clone();
        tokio::spawn(async move {
            if let Err(error) = daemon.handle_connection(stream).await {
                tracing::debug!(error = %error, "connection terminated");
            }
        });
    }

    async fn handle_connection<S>(
        &self,
        stream: S,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
    where
        S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
    {
        let websocket = tokio_tungstenite::accept_hdr_async(stream, negotiate_subprotocol).await?;
        let (connection, receiver) = self.register_connection();
        let (mut sink, mut incoming) = websocket.split();

        let writer = tokio::spawn(async move {
            let mut receiver = receiver;
            while let Some(message) = receiver.recv().await {
                if sink.send(message).await.is_err() {
                    break;
                }
            }
            let _ = sink.close().await;
        });

        while let Some(message) = incoming.next().await {
            match message? {
                Message::Text(text) => self.dispatch(&connection, text.as_str()),
                Message::Close(_) => break,
                _ => {}
            }
        }

        self.unregister_connection(&connection);
        writer.abort();
        Ok(())
    }

    fn register_connection(&self) -> (ConnectionId, mpsc::UnboundedReceiver<Message>) {
        let mut inner = self.state();
        let connection = ConnectionId(new_connection_id(inner.next_connection));
        inner.next_connection += 1;

        let (sender, receiver) = mpsc::unbounded_channel();
        inner.senders.insert(connection.0.clone(), sender);
        (connection, receiver)
    }

    fn unregister_connection(&self, connection: &ConnectionId) {
        let mut inner = self.state();
        inner.senders.remove(connection.as_str());
        inner.broker.unsubscribe(connection);
    }

    fn state(&self) -> std::sync::MutexGuard<'_, Inner> {
        self.inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn send_direct(&self, connection: &ConnectionId, event: CloudEvent) {
        let inner = self.state();
        send_to(&inner, connection, event);
    }

    fn reject(
        &self,
        connection: &ConnectionId,
        command_id: &str,
        error: &(impl std::fmt::Display + ?Sized),
    ) {
        self.reject_with(connection, command_id, event::CODE_COMMAND_FAILED, error);
    }

    fn reject_with(
        &self,
        connection: &ConnectionId,
        command_id: &str,
        code: &str,
        error: &(impl std::fmt::Display + ?Sized),
    ) {
        let inner = self.state();
        let rejection = self
            .encoder
            .command_rejected(command_id, code, &error.to_string());
        send_to(&inner, connection, rejection);
    }

    fn reject_into(
        &self,
        inner: &Inner,
        connection: &ConnectionId,
        command_id: &str,
        error: &(impl std::fmt::Display + ?Sized),
    ) {
        let rejection = self.encoder.command_rejected(
            command_id,
            event::CODE_COMMAND_FAILED,
            &error.to_string(),
        );
        send_to(inner, connection, rejection);
    }
}

fn offload<T>(work: impl FnOnce() -> T) -> T {
    match tokio::runtime::Handle::try_current() {
        Ok(handle) if handle.runtime_flavor() == tokio::runtime::RuntimeFlavor::MultiThread => {
            tokio::task::block_in_place(work)
        }
        _ => work(),
    }
}

#[derive(Debug, thiserror::Error)]
enum RunResolveError {
    #[error("agent '{0}' not found")]
    UnknownAgent(String),
    #[error("model '{0}' not found")]
    UnknownModel(String),
    #[error("provider '{0}' not found")]
    UnknownProvider(String),
    #[error("no credential configured for provider '{0}'")]
    MissingCredential(String),
    #[error("tool '{0}' not found")]
    UnknownTool(String),
    #[error("agent '{agent}' has no instance in project '{project}'")]
    UnknownInstance { agent: String, project: String },
    #[error("workspace '{0}' not found")]
    UnknownWorkspace(String),
    #[error("could not read agents: {0}")]
    Agents(#[from] agent::AgentError),
    #[error("could not read workspaces: {0}")]
    Workspaces(#[from] workspace::WorkspaceError),
}

impl RunResolveError {
    fn code(&self) -> &'static str {
        match self {
            RunResolveError::UnknownAgent(_)
            | RunResolveError::UnknownModel(_)
            | RunResolveError::UnknownProvider(_)
            | RunResolveError::MissingCredential(_)
            | RunResolveError::UnknownTool(_)
            | RunResolveError::UnknownInstance { .. }
            | RunResolveError::UnknownWorkspace(_) => event::CODE_NOT_FOUND,
            RunResolveError::Agents(_) | RunResolveError::Workspaces(_) => event::CODE_INTERNAL,
        }
    }
}

struct PreparedRun {
    endpoint: String,
    api_key: String,
    model: String,
    system: String,
    input: String,
    tools: Vec<ChatTool>,
    scope: RunScope,
}

fn definition_view(definition: AgentDefinition, tools: &[Tool]) -> event::AgentDefinitionView {
    event::AgentDefinitionView {
        id: definition.id.0,
        name: definition.name,
        role: definition.role,
        prompt: definition.prompt,
        tools: tools.iter().map(wire_tool).collect(),
        model: definition.model,
        version: definition.version,
    }
}

fn wire_tool(tool: &Tool) -> protocol::ToolView {
    protocol::ToolView {
        id: tool.id.as_str().to_owned(),
        name: tool.name.clone(),
        description: tool.description.clone(),
        parameters: tool.parameters.clone(),
    }
}

fn instance_view(instance: AgentInstance) -> event::AgentInstanceView {
    event::AgentInstanceView {
        id: instance.id.0,
        agent: instance.agent.0,
        workspace: instance.workspace,
        project: instance.project,
    }
}

fn instance_summary(record: AgentInstanceRecord) -> event::AgentInstanceSummary {
    event::AgentInstanceSummary {
        id: record.id.0,
        agent: record.agent.0,
        workspace: record.workspace,
        project: record.project,
        created_at: record.created_at,
    }
}

fn project_subject(workspace: &str, id: &str) -> String {
    format!("project:{workspace}/{id}")
}

fn project_view(project: Project) -> event::ProjectView {
    event::ProjectView {
        id: project.id.0,
        workspace: project.workspace,
        name: project.name,
    }
}

fn project_summary(record: ProjectRecord) -> event::ProjectSummary {
    event::ProjectSummary {
        id: record.id.0,
        workspace: record.workspace,
        name: record.name,
        created_at: record.created_at,
    }
}

fn publish_domain_event(
    inner: &mut Inner,
    encoder: &Encoder,
    domain_event: DomainEvent,
    correlation: Option<&str>,
) {
    match domain_event {
        DomainEvent::TaskAccepted {
            task_id,
            execution_id,
            at,
        } => {
            let subject = format!("task:{}", task_id.as_str());
            let mut event = encoder.task_accepted(
                &subject,
                task_id.as_str(),
                execution_id.as_str(),
                correlation,
            );
            event.time = Some(at);
            publish(inner, &subject, event);
        }
        DomainEvent::StateChanged {
            execution_id,
            from,
            to,
            at,
        } => {
            let subject = format!("execution:{}", execution_id.as_str());
            let mut event =
                encoder.state_changed(&subject, execution_id.as_str(), from.as_str(), to.as_str());
            event.time = Some(at);
            publish(inner, &subject, event);
        }
        DomainEvent::Log {
            execution_id,
            stream,
            line,
            at,
        } => {
            let subject = format!("execution:{}", execution_id.as_str());
            let mut event =
                encoder.execution_log(&subject, execution_id.as_str(), stream.as_str(), &line);
            event.time = Some(at);
            publish(inner, &subject, event);
        }
    }
}

fn publish(inner: &mut Inner, subject: &str, mut event: CloudEvent) {
    let sequence = inner.broker.reserve_sequence(subject);
    event = event.with_sequence(sequence);
    let payload = serde_json::to_vec(&event).expect("cloud event serializes");

    let recipients = inner.broker.commit(subject, sequence, payload.clone());
    for recipient in recipients {
        if let Some(sender) = inner.senders.get(recipient.as_str()) {
            let _ = sender.send(message_from(payload.clone()));
        }
    }
}

fn send_to(inner: &Inner, connection: &ConnectionId, event: CloudEvent) {
    let payload = match serde_json::to_vec(&event) {
        Ok(payload) => payload,
        Err(_) => return,
    };
    if let Some(sender) = inner.senders.get(connection.as_str()) {
        let _ = sender.send(message_from(payload));
    }
}

fn send_payload(inner: &Inner, connection: &ConnectionId, payload: Vec<u8>) {
    if let Some(sender) = inner.senders.get(connection.as_str()) {
        let _ = sender.send(message_from(payload));
    }
}

fn message_from(payload: Vec<u8>) -> Message {
    match String::from_utf8(payload) {
        Ok(text) => Message::text(text),
        Err(error) => Message::binary(error.into_bytes()),
    }
}

fn bind_unix(socket_path: &Path) -> std::io::Result<UnixListener> {
    if socket_path.exists() {
        std::fs::remove_file(socket_path)?;
    }
    if let Some(parent) = socket_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    UnixListener::bind(socket_path)
}

#[allow(clippy::result_large_err)]
fn negotiate_subprotocol(
    request: &Request,
    mut response: Response,
) -> Result<Response, ErrorResponse> {
    let offered = request
        .headers()
        .get("Sec-WebSocket-Protocol")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();

    let supported = offered
        .split(',')
        .any(|candidate| candidate.trim() == types::SUBPROTOCOL);

    if !supported {
        let mut rejection = ErrorResponse::new(Some(format!(
            "unsupported subprotocol, expected {}",
            types::SUBPROTOCOL
        )));
        *rejection.status_mut() = StatusCode::BAD_REQUEST;
        return Err(rejection);
    }

    response.headers_mut().insert(
        "Sec-WebSocket-Protocol",
        HeaderValue::from_static(types::SUBPROTOCOL),
    );
    Ok(response)
}
