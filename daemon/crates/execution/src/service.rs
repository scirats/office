use std::collections::HashMap;

use crate::domain::{DomainEvent, ExecutionId, ExecutionState, LogStream, TaskId};
use crate::ports::{Clock, IdGenerator};

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

    executions: HashMap<String, ExecutionState>,
}

impl<C: Clock, I: IdGenerator> ExecutionService<C, I> {
    pub fn new(clock: C, ids: I) -> Self {
        Self {
            clock,
            ids,
            executions: HashMap::new(),
        }
    }

    pub fn accept_task(&mut self) -> Vec<DomainEvent> {
        let now = self.clock.now_rfc3339();
        let task_id = TaskId(self.ids.task_id());
        let execution_id = ExecutionId(self.ids.execution_id());

        self.executions
            .insert(execution_id.0.clone(), ExecutionState::Created);

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
        let state = self
            .executions
            .get_mut(execution_id.as_str())
            .ok_or_else(|| ExecutionError::NotFound(execution_id.0.clone()))?;

        let from = *state;
        if !from.can_transition_to(to) {
            return Err(ExecutionError::InvalidTransition {
                from: from.as_str(),
                to: to.as_str(),
            });
        }
        *state = to;

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
        let events = service.accept_task();
        match &events[0] {
            DomainEvent::TaskAccepted { execution_id, .. } => execution_id.clone(),
            other => panic!("expected TaskAccepted, got {other:?}"),
        }
    }

    #[test]
    fn accept_task_creates_execution_in_created_state() {
        let mut service = service();
        let id = accept(&mut service);

        let events = service
            .transition(&id, ExecutionState::Preparing)
            .expect("transitions");
        match &events[0] {
            DomainEvent::StateChanged { from, to, .. } => {
                assert_eq!(*from, ExecutionState::Created);
                assert_eq!(*to, ExecutionState::Preparing);
            }
            other => panic!("expected StateChanged, got {other:?}"),
        }
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
