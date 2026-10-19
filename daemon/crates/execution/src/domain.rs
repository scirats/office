#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExecutionId(pub String);

impl ExecutionId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ExecutionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TaskId(pub String);

impl TaskId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for TaskId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionState {
    Created,
    Preparing,
    Ready,
    Running,
    Finishing,
    Succeeded,
    Failed,
    Cancelled,
    Orphaned,
}

impl ExecutionState {
    pub fn as_str(&self) -> &'static str {
        match self {
            ExecutionState::Created => "CREATED",
            ExecutionState::Preparing => "PREPARING",
            ExecutionState::Ready => "READY",
            ExecutionState::Running => "RUNNING",
            ExecutionState::Finishing => "FINISHING",
            ExecutionState::Succeeded => "SUCCEEDED",
            ExecutionState::Failed => "FAILED",
            ExecutionState::Cancelled => "CANCELLED",
            ExecutionState::Orphaned => "ORPHANED",
        }
    }

    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            ExecutionState::Succeeded
                | ExecutionState::Failed
                | ExecutionState::Cancelled
                | ExecutionState::Orphaned
        )
    }

    pub fn can_transition_to(&self, next: ExecutionState) -> bool {
        use ExecutionState::*;
        matches!(
            (self, next),
            (Created, Preparing)
                | (Created, Cancelled)
                | (Preparing, Ready)
                | (Preparing, Failed)
                | (Preparing, Cancelled)
                | (Ready, Running)
                | (Ready, Cancelled)
                | (Running, Finishing)
                | (Running, Failed)
                | (Running, Cancelled)
                | (Finishing, Succeeded)
                | (Finishing, Failed)
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogStream {
    Stdout,
    Stderr,

    Tool,

    ToolResult,
}

impl LogStream {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogStream::Stdout => "stdout",
            LogStream::Stderr => "stderr",
            LogStream::Tool => "tool",
            LogStream::ToolResult => "tool-result",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainEvent {
    TaskAccepted {
        task_id: TaskId,
        execution_id: ExecutionId,
        at: String,
    },
    StateChanged {
        execution_id: ExecutionId,
        from: ExecutionState,
        to: ExecutionState,
        at: String,
    },
    Log {
        execution_id: ExecutionId,
        stream: LogStream,
        line: String,
        at: String,
    },
}

#[cfg(test)]
mod tests {
    use super::ExecutionState::*;

    #[test]
    fn allows_happy_path() {
        let path = [Created, Preparing, Ready, Running, Finishing, Succeeded];
        for window in path.windows(2) {
            assert!(
                window[0].can_transition_to(window[1]),
                "{} -> {} should be allowed",
                window[0].as_str(),
                window[1].as_str()
            );
        }
    }

    #[test]
    fn rejects_skipping_states() {
        assert!(!Created.can_transition_to(Running));
        assert!(!Preparing.can_transition_to(Succeeded));
        assert!(!Ready.can_transition_to(Finishing));
    }

    #[test]
    fn terminal_states_have_no_transitions() {
        for state in [Succeeded, Failed, Cancelled, Orphaned] {
            assert!(state.is_terminal());
            for next in [Created, Preparing, Ready, Running, Finishing, Succeeded] {
                assert!(!state.can_transition_to(next));
            }
        }
    }

    #[test]
    fn cancellation_is_only_reachable_before_finishing() {
        assert!(Created.can_transition_to(Cancelled));
        assert!(Running.can_transition_to(Cancelled));
        assert!(!Finishing.can_transition_to(Cancelled));
    }
}
