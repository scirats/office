use agent::InstanceIdSource;
use execution::IdGenerator;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Default)]
pub struct UuidV7Generator;

impl IdGenerator for UuidV7Generator {
    fn execution_id(&self) -> String {
        format!("exec_{}", Uuid::now_v7().simple())
    }

    fn task_id(&self) -> String {
        format!("task_{}", Uuid::now_v7().simple())
    }
}

impl InstanceIdSource for UuidV7Generator {
    fn instance_id(&self) -> String {
        format!("inst_{}", Uuid::now_v7().simple())
    }
}

pub fn new_event_id() -> String {
    format!("evt_{}", Uuid::now_v7().simple())
}

pub fn new_connection_id(sequence: u64) -> String {
    format!("conn_{sequence}")
}
