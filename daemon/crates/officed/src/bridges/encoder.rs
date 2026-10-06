use execution::Clock;
use protocol::{CloudEvent, event, types};
use serde::Serialize;

use crate::adapters::{SystemClock, new_event_id};

const DAEMON_SOURCE: &str = "/office/daemon";

#[derive(Debug, Clone, Copy, Default)]
pub struct Encoder;

impl Encoder {
    fn daemon_event(&self, type_: &str, subject: Option<&str>, data: impl Serialize) -> CloudEvent {
        let payload = serde_json::to_value(data).expect("outgoing payload is serializable");
        let mut event = CloudEvent::new(new_event_id(), DAEMON_SOURCE, type_, payload);
        event.time = Some(SystemClock.now_rfc3339());
        event.subject = subject.map(str::to_owned);
        event
    }

    pub fn command_accepted(&self, command_id: &str, subject: Option<&str>) -> CloudEvent {
        self.daemon_event(
            types::event::COMMAND_ACCEPTED,
            subject,
            event::CommandAccepted {
                command_id: command_id.to_owned(),
            },
        )
        .with_correlation(command_id)
    }

    pub fn command_rejected(&self, command_id: &str, code: &str, message: &str) -> CloudEvent {
        self.daemon_event(
            types::event::COMMAND_REJECTED,
            None,
            event::CommandRejected {
                command_id: command_id.to_owned(),
                code: code.to_owned(),
                message: message.to_owned(),
            },
        )
        .with_correlation(command_id)
    }

    pub fn task_accepted(
        &self,
        subject: &str,
        task_id: &str,
        execution_id: &str,
        correlation: Option<&str>,
    ) -> CloudEvent {
        let event = self.daemon_event(
            types::event::TASK_ACCEPTED,
            Some(subject),
            event::TaskAccepted {
                task_id: task_id.to_owned(),
                execution_id: execution_id.to_owned(),
            },
        );
        match correlation {
            Some(correlation) => event.with_correlation(correlation),
            None => event,
        }
    }

    pub fn state_changed(
        &self,
        subject: &str,
        execution_id: &str,
        from: &str,
        to: &str,
    ) -> CloudEvent {
        self.daemon_event(
            types::event::EXECUTION_STATE_CHANGED,
            Some(subject),
            event::ExecutionStateChanged {
                execution_id: execution_id.to_owned(),
                from: from.to_owned(),
                to: to.to_owned(),
            },
        )
    }

    pub fn execution_log(
        &self,
        subject: &str,
        execution_id: &str,
        stream: &str,
        line: &str,
    ) -> CloudEvent {
        self.daemon_event(
            types::event::EXECUTION_LOG,
            Some(subject),
            event::ExecutionLog {
                execution_id: execution_id.to_owned(),
                stream: stream.to_owned(),
                line: line.to_owned(),
            },
        )
    }

    pub fn office_error(&self, code: &str, message: &str, subject: Option<&str>) -> CloudEvent {
        self.daemon_event(
            types::event::ERROR,
            subject,
            event::OfficeError {
                code: code.to_owned(),
                message: message.to_owned(),
            },
        )
    }

    pub fn workspace_created(
        &self,
        subject: &str,
        payload: event::WorkspaceCreated,
        correlation: Option<&str>,
    ) -> CloudEvent {
        let created = self.daemon_event(types::event::WORKSPACE_CREATED, Some(subject), payload);
        match correlation {
            Some(value) => created.with_correlation(value),
            None => created,
        }
    }

    pub fn workspace_listed(
        &self,
        subject: Option<&str>,
        payload: event::WorkspaceListed,
    ) -> CloudEvent {
        self.daemon_event(types::event::WORKSPACE_LISTED, subject, payload)
    }

    pub fn workspace_status_changed(
        &self,
        subject: &str,
        payload: event::WorkspaceStatusChanged,
    ) -> CloudEvent {
        self.daemon_event(
            types::event::WORKSPACE_STATUS_CHANGED,
            Some(subject),
            payload,
        )
    }

    pub fn agent_definition_created(
        &self,
        subject: &str,
        payload: event::AgentDefinitionView,
        correlation: Option<&str>,
    ) -> CloudEvent {
        let created = self.daemon_event(
            types::event::AGENT_DEFINITION_CREATED,
            Some(subject),
            payload,
        );
        match correlation {
            Some(value) => created.with_correlation(value),
            None => created,
        }
    }

    pub fn agent_definition_listed(
        &self,
        subject: Option<&str>,
        payload: event::AgentDefinitionListed,
    ) -> CloudEvent {
        self.daemon_event(types::event::AGENT_DEFINITION_LISTED, subject, payload)
    }

    pub fn agent_instance_created(
        &self,
        subject: &str,
        payload: event::AgentInstanceView,
        correlation: Option<&str>,
    ) -> CloudEvent {
        let created =
            self.daemon_event(types::event::AGENT_INSTANCE_CREATED, Some(subject), payload);
        match correlation {
            Some(value) => created.with_correlation(value),
            None => created,
        }
    }

    pub fn agent_instance_listed(
        &self,
        subject: Option<&str>,
        payload: event::AgentInstanceListed,
    ) -> CloudEvent {
        self.daemon_event(types::event::AGENT_INSTANCE_LISTED, subject, payload)
    }

    pub fn agent_instance_deleted(
        &self,
        subject: &str,
        payload: event::AgentInstanceDeleted,
    ) -> CloudEvent {
        self.daemon_event(types::event::AGENT_INSTANCE_DELETED, Some(subject), payload)
    }

    pub fn tool_listed(&self, subject: Option<&str>, payload: event::ToolListed) -> CloudEvent {
        self.daemon_event(types::event::TOOL_LISTED, subject, payload)
    }
}
