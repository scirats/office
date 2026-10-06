use std::collections::HashMap;

use crate::domain::{DomainEvent, ExecutionId, ExecutionState, LogStream, TaskId};
use crate::ports::{Clock, IdGenerator};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcceptTask {
    pub company: String,
    pub project: String,
    pub agent: String,
    pub input: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Execution {
    pub id: ExecutionId,
    pub task_id: TaskId,
    pub state: ExecutionState,
    pub company: String,
    pub project: String,
    pub agent: String,
    pub input: String,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ExecutionError {
    #[error("execution not found: {0}")]
    NotFound(String),
    #[error("invalid transition: {from} -> {to}")]
    InvalidTransition {
        from: &'static str,
        to: &'static str,
    },
}

pub struct ExecutionService<C: Clock, I: IdGenerator> {
    clock: C,
    ids: I,
    executions: HashMap<String, Execution>,
}

impl<C: Clock, I: IdGenerator> ExecutionService<C, I> {
    pub fn new(clock: C, ids: I) -> Self {
        Self {
            clock,
            ids,
            executions: HashMap::new(),
        }
    }

    pub fn accept_task(&mut self, command: AcceptTask) -> Vec<DomainEvent> {
        let now = self.clock.now_rfc3339();
        let task_id = TaskId(self.ids.task_id());
        let execution_id = ExecutionId(self.ids.execution_id());

        let execution = Execution {
            id: execution_id.clone(),
            task_id: task_id.clone(),
            state: ExecutionState::Created,
            company: command.company,
            project: command.project,
            agent: command.agent,
            input: command.input,
        };
        self.executions.insert(execution_id.0.clone(), execution);

        vec![DomainEvent::TaskAccepted {
            task_id,
            execution_id,
            at: now,
        }]
    }

    pub fn transition(
        &mut self,
        execution_id: &ExecutionId,
        to: ExecutionState,
    ) -> Result<Vec<DomainEvent>, ExecutionError> {
        let now = self.clock.now_rfc3339();
        let execution = self
            .executions
            .get_mut(execution_id.as_str())
            .ok_or_else(|| ExecutionError::NotFound(execution_id.0.clone()))?;

        let from = execution.state;
        if !from.can_transition_to(to) {
            return Err(ExecutionError::InvalidTransition {
                from: from.as_str(),
                to: to.as_str(),
            });
        }
        execution.state = to;

        Ok(vec![DomainEvent::StateChanged {
            execution_id: execution_id.clone(),
            from,
            to,
            at: now,
        }])
    }

    pub fn cancel(
        &mut self,
        execution_id: &ExecutionId,
    ) -> Result<Vec<DomainEvent>, ExecutionError> {
        self.transition(execution_id, ExecutionState::Cancelled)
    }

    pub fn log(
        &mut self,
        execution_id: &ExecutionId,
        stream: LogStream,
        line: impl Into<String>,
    ) -> Result<Vec<DomainEvent>, ExecutionError> {
        let now = self.clock.now_rfc3339();
        self.executions
            .get(execution_id.as_str())
            .ok_or_else(|| ExecutionError::NotFound(execution_id.0.clone()))?;

        Ok(vec![DomainEvent::Log {
            execution_id: execution_id.clone(),
            stream,
            line: line.into(),
            at: now,
        }])
    }

    pub fn get(&self, execution_id: &ExecutionId) -> Option<&Execution> {
        self.executions.get(execution_id.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct FixedClock;
    impl Clock for FixedClock {
        fn now_rfc3339(&self) -> String {
            "2026-10-05T12:00:00Z".to_owned()
        }
    }

    #[derive(Default)]
    struct SequentialIds(std::sync::Mutex<u64>);
    impl IdGenerator for SequentialIds {
        fn execution_id(&self) -> String {
            let mut n = self.0.lock().unwrap();
            *n += 1;
            format!("exec_{n}")
        }
        fn task_id(&self) -> String {
            format!("task_{}", self.execution_id())
        }
    }

    fn service() -> ExecutionService<FixedClock, SequentialIds> {
        ExecutionService::new(FixedClock, SequentialIds::default())
    }

    fn accept(service: &mut ExecutionService<FixedClock, SequentialIds>) -> ExecutionId {
        let events = service.accept_task(AcceptTask {
            company: "acme".into(),
            project: "ecommerce".into(),
            agent: "backend-engineer".into(),
            input: "fix checkout".into(),
        });
        match &events[0] {
            DomainEvent::TaskAccepted { execution_id, .. } => execution_id.clone(),
            other => panic!("expected TaskAccepted, got {other:?}"),
        }
    }

    #[test]
    fn accept_task_creates_execution_in_created_state() {
        let mut service = service();
        let id = accept(&mut service);
        assert_eq!(service.get(&id).unwrap().state, ExecutionState::Created);
    }

    #[test]
    fn enforces_transitions() {
        let mut service = service();
        let id = accept(&mut service);

        service.transition(&id, ExecutionState::Preparing).unwrap();
        let error = service
            .transition(&id, ExecutionState::Succeeded)
            .unwrap_err();
        assert_eq!(
            error,
            ExecutionError::InvalidTransition {
                from: "PREPARING",
                to: "SUCCEEDED"
            }
        );
    }

    #[test]
    fn cannot_cancel_after_finishing() {
        let mut service = service();
        let id = accept(&mut service);
        for state in [
            ExecutionState::Preparing,
            ExecutionState::Ready,
            ExecutionState::Running,
            ExecutionState::Finishing,
            ExecutionState::Succeeded,
        ] {
            service.transition(&id, state).unwrap();
        }
        assert!(service.cancel(&id).is_err());
    }

    #[test]
    fn transition_on_unknown_execution_fails() {
        let mut service = service();
        let error = service
            .transition(&ExecutionId("missing".into()), ExecutionState::Preparing)
            .unwrap_err();
        assert_eq!(error, ExecutionError::NotFound("missing".into()));
    }
}
