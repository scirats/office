pub trait Clock: Send + Sync {
    fn now_rfc3339(&self) -> String;
}

pub trait IdGenerator: Send + Sync {
    fn execution_id(&self) -> String;
    fn task_id(&self) -> String;
}
