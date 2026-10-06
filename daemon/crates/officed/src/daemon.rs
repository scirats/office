use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use agent::{
    AgentDefinition, AgentInstance, AgentInstanceRecord, AgentRepository, AgentService, DefineAgent,
};
use broker::{Broker, ConnectionId, Filter, ReplayError};
use execution::{
    AcceptTask, DomainEvent, ExecutionId, ExecutionService, ExecutionState, LogStream,
};
use futures_util::{SinkExt, StreamExt};
use protocol::command::{
    AgentDefinitionCreate, AgentInstanceCreate, AgentInstanceDelete, AgentInstanceList,
};
use protocol::event;
use protocol::types;
use protocol::{CloudEvent, Command};
use reconciliation::{ExpectedWorkspace, ReconciliationService, WorkspaceInspector};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::{TcpListener, UnixListener};
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::handshake::server::{ErrorResponse, Request, Response};
use tokio_tungstenite::tungstenite::http::{HeaderValue, StatusCode};
use tool::{Tool, ToolService};
use workspace::{UserProvisioner, WorkspaceRepository, WorkspaceService};

use crate::adapters::{PlatformUserProvisioner, SystemClock, UuidV7Generator, new_connection_id};
use crate::bridges::Encoder;
use toolbox::BuiltinToolCatalog;

const DEFAULT_BROKER_CAPACITY: usize = 1024;
const DEFAULT_RECONCILE_SECS: u64 = 30;

#[derive(Clone)]
pub struct Daemon {
    inner: Arc<Mutex<Inner>>,
    workspaces: Arc<WorkspaceService>,
    agents: Arc<AgentService>,
    tools: Arc<ToolService>,
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
        inspector: Box<dyn WorkspaceInspector>,
    ) -> Self {
        Self::with_services(
            DEFAULT_BROKER_CAPACITY,
            Box::new(PlatformUserProvisioner),
            workspaces,
            agents,
            inspector,
        )
    }

    pub fn with_services(
        capacity: usize,
        provisioner: Box<dyn UserProvisioner>,
        workspaces: Box<dyn WorkspaceRepository>,
        agents: Box<dyn AgentRepository>,
        inspector: Box<dyn WorkspaceInspector>,
    ) -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner {
                executions: ExecutionService::new(SystemClock, UuidV7Generator),
                broker: Broker::new(capacity),
                senders: HashMap::new(),
                next_connection: 1,
            })),
            workspaces: Arc::new(WorkspaceService::new(provisioner, workspaces)),
            agents: Arc::new(AgentService::new(agents, Box::new(UuidV7Generator))),
            tools: Arc::new(ToolService::new(Box::new(BuiltinToolCatalog))),
            reconciler: Arc::new(ReconciliationService::new(inspector)),
            reconcile_interval: Duration::from_secs(DEFAULT_RECONCILE_SECS),
            encoder: Encoder,
        }
    }

    pub fn with_reconcile_interval(mut self, interval: Duration) -> Self {
        self.reconcile_interval = interval;
        self
    }

    pub fn reconcile(&self) {
        let records = match self.workspaces.list() {
            Ok(records) => records,
            Err(error) => {
                eprintln!("office: reconciliation failed to read workspaces: {error}");
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

        let report = self.reconciler.run(&expected);

        for reconciliation in &report.workspaces {
            let status = reconciliation.status.as_str();
            if previous.get(&reconciliation.slug).map(String::as_str) == Some(status) {
                continue;
            }

            if let Err(error) = self.workspaces.set_status(&reconciliation.slug, status) {
                eprintln!(
                    "office: could not persist status for '{}': {error}",
                    reconciliation.slug
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
                eprintln!("office: workspace '{}' is active", reconciliation.slug);
            } else {
                eprintln!(
                    "office: workspace '{}' is {} ({})",
                    reconciliation.slug,
                    status,
                    payload.inconsistencies.join(", ")
                );
            }

            let status_event = self.encoder.workspace_status_changed(&subject, payload);
            let mut inner = self.inner.lock().expect("daemon state poisoned");
            publish(&mut inner, &subject, status_event);
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
                Err(error) => eprintln!("office: unix accept failed: {error}"),
            }
        }
    }

    async fn accept_tcp(self, listener: TcpListener) -> std::io::Result<()> {
        loop {
            match listener.accept().await {
                Ok((stream, _)) => self.spawn_connection(stream),
                Err(error) => eprintln!("office: tcp accept failed: {error}"),
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
                eprintln!("office: connection terminated: {error}");
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
        let mut inner = self.inner.lock().expect("daemon state poisoned");
        let connection = ConnectionId(new_connection_id(inner.next_connection));
        inner.next_connection += 1;

        let (sender, receiver) = mpsc::unbounded_channel();
        inner.senders.insert(connection.0.clone(), sender);
        (connection, receiver)
    }

    fn unregister_connection(&self, connection: &ConnectionId) {
        let mut inner = self.inner.lock().expect("daemon state poisoned");
        inner.senders.remove(connection.as_str());
        inner.broker.unsubscribe(connection);
    }

    fn dispatch(&self, connection: &ConnectionId, text: &str) {
        let event = match CloudEvent::from_json(text) {
            Ok(event) => event,
            Err(error) => {
                let report =
                    self.encoder
                        .office_error(event::CODE_INVALID_EVENT, &error.to_string(), None);
                self.send_direct(connection, report);
                return;
            }
        };

        let command = match Command::from_event(&event) {
            Ok(command) => command,
            Err(error) => {
                let rejection = self.encoder.command_rejected(
                    &event.id,
                    event::CODE_UNKNOWN_COMMAND,
                    &error.to_string(),
                );
                self.send_direct(connection, rejection);
                return;
            }
        };

        match command {
            Command::WorkspaceCreate(create) => {
                self.create_workspace(connection, &event, &create.name)
            }
            Command::WorkspaceList(_) => self.list_workspaces(connection, &event),
            Command::AgentDefinitionCreate(create) => {
                self.create_agent_definition(connection, &event, create)
            }
            Command::AgentDefinitionList(_) => self.list_agent_definitions(connection, &event),
            Command::AgentInstanceCreate(create) => {
                self.link_agent_instance(connection, &event, create)
            }
            Command::AgentInstanceList(list) => self.list_agent_instances(connection, &event, list),
            Command::AgentInstanceDelete(delete) => {
                self.unlink_agent_instance(connection, &event, delete)
            }
            Command::ToolList(_) => self.list_tools(connection, &event),
            Command::SubscriptionAdd(added) => {
                let mut inner = self.inner.lock().expect("daemon state poisoned");
                inner.broker.subscribe(
                    connection.clone(),
                    Filter {
                        subject_prefix: added.filter.subject_prefix,
                    },
                );
                let accepted = self
                    .encoder
                    .command_accepted(&event.id, event.subject.as_deref());
                send_to(&inner, connection, accepted);
            }
            Command::SubscriptionRemove(_) => {
                let mut inner = self.inner.lock().expect("daemon state poisoned");
                inner.broker.unsubscribe(connection);
                let accepted = self
                    .encoder
                    .command_accepted(&event.id, event.subject.as_deref());
                send_to(&inner, connection, accepted);
            }
            Command::SubscriptionResume(resume) => {
                let inner = self.inner.lock().expect("daemon state poisoned");
                match inner.broker.replay(&resume.subject, resume.last_sequence) {
                    Ok(stored) => {
                        let accepted = self
                            .encoder
                            .command_accepted(&event.id, Some(&resume.subject));
                        send_to(&inner, connection, accepted);
                        for stored_event in stored {
                            send_payload(&inner, connection, stored_event.payload);
                        }
                    }
                    Err(ReplayError::ResyncRequired { subject, oldest }) => {
                        let message = format!(
                            "cursor {} is older than retained sequence {oldest}",
                            resume.last_sequence
                        );
                        let report = self.encoder.office_error(
                            event::CODE_RESYNC_REQUIRED,
                            &message,
                            Some(&subject),
                        );
                        send_to(&inner, connection, report);
                    }
                }
            }
            Command::TaskSubmit(submit) => {
                let mut inner = self.inner.lock().expect("daemon state poisoned");
                let domain_events = inner.executions.accept_task(AcceptTask {
                    company: submit.company,
                    project: submit.project,
                    agent: submit.agent,
                    input: submit.input,
                });
                let accepted = self
                    .encoder
                    .command_accepted(&event.id, event.subject.as_deref());
                send_to(&inner, connection, accepted);

                let mut execution_id = None;
                for domain_event in domain_events {
                    if let DomainEvent::TaskAccepted {
                        execution_id: id, ..
                    } = &domain_event
                    {
                        execution_id = Some(id.clone());
                    }
                    publish_domain_event(&mut inner, &self.encoder, domain_event, Some(&event.id));
                }

                if let Some(execution_id) = execution_id {
                    if let Ok(events) = inner
                        .executions
                        .transition(&execution_id, ExecutionState::Preparing)
                    {
                        for domain_event in events {
                            publish_domain_event(
                                &mut inner,
                                &self.encoder,
                                domain_event,
                                Some(&event.id),
                            );
                        }
                    }

                    let daemon = self.clone();
                    tokio::spawn(async move { daemon.progress(execution_id).await });
                }
            }
            Command::ExecutionCancel(cancel) => {
                let mut inner = self.inner.lock().expect("daemon state poisoned");
                let execution_id = ExecutionId(cancel.execution_id);
                match inner.executions.cancel(&execution_id) {
                    Ok(events) => {
                        let accepted = self
                            .encoder
                            .command_accepted(&event.id, event.subject.as_deref());
                        send_to(&inner, connection, accepted);
                        for domain_event in events {
                            publish_domain_event(
                                &mut inner,
                                &self.encoder,
                                domain_event,
                                Some(&event.id),
                            );
                        }
                    }
                    Err(error) => {
                        let rejection = self.encoder.command_rejected(
                            &event.id,
                            event::CODE_COMMAND_FAILED,
                            &error.to_string(),
                        );
                        send_to(&inner, connection, rejection);
                    }
                }
            }
        }
    }

    fn create_workspace(&self, connection: &ConnectionId, event: &CloudEvent, name: &str) {
        let result = self.workspaces.create(name);
        let mut inner = self.inner.lock().expect("daemon state poisoned");

        match result {
            Ok(workspace) => {
                let accepted = self
                    .encoder
                    .command_accepted(&event.id, event.subject.as_deref());
                send_to(&inner, connection, accepted);

                let subject = format!("workspace:{}", workspace.slug);
                let payload = event::WorkspaceCreated {
                    name: workspace.name,
                    slug: workspace.slug,
                    user: workspace.user,
                    home: workspace.home,
                };
                let created = self
                    .encoder
                    .workspace_created(&subject, payload, Some(&event.id));
                publish(&mut inner, &subject, created);
            }
            Err(error) => {
                let rejection = self.encoder.command_rejected(
                    &event.id,
                    event::CODE_COMMAND_FAILED,
                    &error.to_string(),
                );
                send_to(&inner, connection, rejection);
            }
        }
    }

    fn list_workspaces(&self, connection: &ConnectionId, event: &CloudEvent) {
        let result = self.workspaces.list();
        let inner = self.inner.lock().expect("daemon state poisoned");

        match result {
            Ok(records) => {
                let payload = event::WorkspaceListed {
                    workspaces: records
                        .into_iter()
                        .map(|record| event::WorkspaceSummary {
                            name: record.name,
                            slug: record.slug,
                            user: record.user,
                            home: record.home,
                            created_at: record.created_at,
                            status: record.status,
                        })
                        .collect(),
                };
                let listed = self
                    .encoder
                    .workspace_listed(event.subject.as_deref(), payload);
                send_to(&inner, connection, listed);
            }
            Err(error) => {
                let rejection = self.encoder.command_rejected(
                    &event.id,
                    event::CODE_COMMAND_FAILED,
                    &error.to_string(),
                );
                send_to(&inner, connection, rejection);
            }
        }
    }

    fn create_agent_definition(
        &self,
        connection: &ConnectionId,
        event: &CloudEvent,
        create: AgentDefinitionCreate,
    ) {
        let resolved = match self.tools.resolve(&create.tools) {
            Ok(tools) => tools,
            Err(error) => {
                let inner = self.inner.lock().expect("daemon state poisoned");
                let rejection = self.encoder.command_rejected(
                    &event.id,
                    event::CODE_COMMAND_FAILED,
                    &error.to_string(),
                );
                send_to(&inner, connection, rejection);
                return;
            }
        };

        let result = self.agents.define(DefineAgent {
            name: create.name,
            role: create.role,
            prompt: create.prompt,
            tools: create.tools,
        });
        let mut inner = self.inner.lock().expect("daemon state poisoned");

        match result {
            Ok(definition) => {
                let accepted = self
                    .encoder
                    .command_accepted(&event.id, event.subject.as_deref());
                send_to(&inner, connection, accepted);

                let subject = format!("agent:{}", definition.id.as_str());
                let created = self.encoder.agent_definition_created(
                    &subject,
                    definition_view(definition, &resolved),
                    Some(&event.id),
                );
                publish(&mut inner, &subject, created);
            }
            Err(error) => {
                let rejection = self.encoder.command_rejected(
                    &event.id,
                    event::CODE_COMMAND_FAILED,
                    &error.to_string(),
                );
                send_to(&inner, connection, rejection);
            }
        }
    }

    fn list_agent_definitions(&self, connection: &ConnectionId, event: &CloudEvent) {
        let result = self.agents.list_definitions();
        let inner = self.inner.lock().expect("daemon state poisoned");

        match result {
            Ok(definitions) => {
                let mut views = Vec::with_capacity(definitions.len());
                for definition in definitions {
                    let resolved = match self.tools.resolve(&definition.tools) {
                        Ok(tools) => tools,
                        Err(error) => {
                            let rejection = self.encoder.command_rejected(
                                &event.id,
                                event::CODE_COMMAND_FAILED,
                                &error.to_string(),
                            );
                            send_to(&inner, connection, rejection);
                            return;
                        }
                    };
                    views.push(definition_view(definition, &resolved));
                }
                let listed = self.encoder.agent_definition_listed(
                    event.subject.as_deref(),
                    event::AgentDefinitionListed { definitions: views },
                );
                send_to(&inner, connection, listed);
            }
            Err(error) => {
                let rejection = self.encoder.command_rejected(
                    &event.id,
                    event::CODE_COMMAND_FAILED,
                    &error.to_string(),
                );
                send_to(&inner, connection, rejection);
            }
        }
    }

    fn link_agent_instance(
        &self,
        connection: &ConnectionId,
        event: &CloudEvent,
        create: AgentInstanceCreate,
    ) {
        let result = self
            .agents
            .link(&create.agent, &create.workspace, &create.project);
        let mut inner = self.inner.lock().expect("daemon state poisoned");

        match result {
            Ok(instance) => {
                let accepted = self
                    .encoder
                    .command_accepted(&event.id, event.subject.as_deref());
                send_to(&inner, connection, accepted);

                let subject = format!("agent-instance:{}", instance.id.as_str());
                let created = self.encoder.agent_instance_created(
                    &subject,
                    instance_view(instance),
                    Some(&event.id),
                );
                publish(&mut inner, &subject, created);
            }
            Err(error) => {
                let rejection = self.encoder.command_rejected(
                    &event.id,
                    event::CODE_COMMAND_FAILED,
                    &error.to_string(),
                );
                send_to(&inner, connection, rejection);
            }
        }
    }

    fn list_agent_instances(
        &self,
        connection: &ConnectionId,
        event: &CloudEvent,
        list: AgentInstanceList,
    ) {
        let result = self
            .agents
            .list_instances(list.workspace.as_deref(), list.project.as_deref());
        let inner = self.inner.lock().expect("daemon state poisoned");

        match result {
            Ok(records) => {
                let listed = self.encoder.agent_instance_listed(
                    event.subject.as_deref(),
                    event::AgentInstanceListed {
                        instances: records.into_iter().map(instance_summary).collect(),
                    },
                );
                send_to(&inner, connection, listed);
            }
            Err(error) => {
                let rejection = self.encoder.command_rejected(
                    &event.id,
                    event::CODE_COMMAND_FAILED,
                    &error.to_string(),
                );
                send_to(&inner, connection, rejection);
            }
        }
    }

    fn unlink_agent_instance(
        &self,
        connection: &ConnectionId,
        event: &CloudEvent,
        delete: AgentInstanceDelete,
    ) {
        let result = self
            .agents
            .unlink(&delete.agent, &delete.workspace, &delete.project);
        let mut inner = self.inner.lock().expect("daemon state poisoned");

        match result {
            Ok(removed) => {
                let accepted = self
                    .encoder
                    .command_accepted(&event.id, event.subject.as_deref());
                send_to(&inner, connection, accepted);

                if let Some(instance_id) = removed {
                    let subject = format!("agent-instance:{instance_id}");
                    let deleted = self.encoder.agent_instance_deleted(
                        &subject,
                        event::AgentInstanceDeleted {
                            agent: delete.agent,
                            workspace: delete.workspace,
                            project: delete.project,
                        },
                    );
                    publish(&mut inner, &subject, deleted);
                }
            }
            Err(error) => {
                let rejection = self.encoder.command_rejected(
                    &event.id,
                    event::CODE_COMMAND_FAILED,
                    &error.to_string(),
                );
                send_to(&inner, connection, rejection);
            }
        }
    }

    fn list_tools(&self, connection: &ConnectionId, event: &CloudEvent) {
        let tools = self.tools.list();
        let inner = self.inner.lock().expect("daemon state poisoned");

        let listed = self.encoder.tool_listed(
            event.subject.as_deref(),
            event::ToolListed {
                tools: tools.iter().map(wire_tool).collect(),
            },
        );
        send_to(&inner, connection, listed);
    }

    fn send_direct(&self, connection: &ConnectionId, event: CloudEvent) {
        let inner = self.inner.lock().expect("daemon state poisoned");
        send_to(&inner, connection, event);
    }

    async fn progress(self, execution_id: ExecutionId) {
        for (delay, state, stream, line) in progress_steps() {
            tokio::time::sleep(delay).await;

            let mut inner = self.inner.lock().expect("daemon state poisoned");
            if let Ok(events) = inner.executions.log(&execution_id, stream, line) {
                for domain_event in events {
                    publish_domain_event(&mut inner, &self.encoder, domain_event, None);
                }
            }
            if let Ok(events) = inner.executions.transition(&execution_id, state) {
                for domain_event in events {
                    publish_domain_event(&mut inner, &self.encoder, domain_event, None);
                }
            }
        }
    }
}

fn progress_steps() -> [(Duration, ExecutionState, LogStream, &'static str); 4] {
    [
        (
            Duration::from_millis(40),
            ExecutionState::Ready,
            LogStream::Stdout,
            "environment ready",
        ),
        (
            Duration::from_millis(40),
            ExecutionState::Running,
            LogStream::Stdout,
            "running tests...",
        ),
        (
            Duration::from_millis(80),
            ExecutionState::Finishing,
            LogStream::Stdout,
            "finalizing",
        ),
        (
            Duration::from_millis(40),
            ExecutionState::Succeeded,
            LogStream::Stdout,
            "done",
        ),
    ]
}

fn definition_view(definition: AgentDefinition, tools: &[Tool]) -> event::AgentDefinitionView {
    event::AgentDefinitionView {
        id: definition.id.0,
        name: definition.name,
        role: definition.role,
        prompt: definition.prompt,
        tools: tools.iter().map(wire_tool).collect(),
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
    event.sequence = Some(sequence);
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
