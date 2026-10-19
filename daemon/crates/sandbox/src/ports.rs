use futures_util::future::BoxFuture;

use crate::domain::{SandboxError, SandboxOutput, SandboxSpec};

pub trait Sandbox: Send + Sync {
    fn run(&self, spec: SandboxSpec) -> BoxFuture<'static, Result<SandboxOutput, SandboxError>>;
}
