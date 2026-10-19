use std::path::PathBuf;

use futures_util::future::BoxFuture;
use sandbox::{Sandbox, SandboxError, SandboxOutput, SandboxSpec};

use super::process::{CgroupConfig, capture};

pub struct BwrapSandbox {
    cgroup_root: Option<PathBuf>,
}

impl BwrapSandbox {
    pub fn new(cgroup_root: Option<PathBuf>) -> Self {
        Self { cgroup_root }
    }
}

impl Sandbox for BwrapSandbox {
    fn run(&self, spec: SandboxSpec) -> BoxFuture<'static, Result<SandboxOutput, SandboxError>> {
        let cgroup_root = self.cgroup_root.clone();
        Box::pin(async move {
            let mut command = tokio::process::Command::new("bwrap");
            command.arg("--unshare-all");
            if spec.network {
                command.arg("--share-net");
            }
            command.arg("--die-with-parent").arg("--new-session");
            for mount in &spec.mounts {
                command.arg(if mount.read_only {
                    "--ro-bind"
                } else {
                    "--bind"
                });
                command.arg(&mount.source).arg(&mount.target);
            }
            command.arg("--proc").arg("/proc");
            command.arg("--dev").arg("/dev");
            command.arg("--tmpfs").arg("/tmp");
            command.arg("--chdir").arg(&spec.workdir);
            for (key, value) in &spec.env {
                command.arg("--setenv").arg(key).arg(value);
            }
            command.arg("--").arg(&spec.program).args(&spec.args);

            let cgroup = cgroup_root.map(|root| CgroupConfig {
                root,
                name: spec.execution_id.clone(),
                memory_max_bytes: spec.limits.memory_max_bytes,
                pids_max: spec.limits.pids_max,
            });
            capture(command, spec.timeout, cgroup).await
        })
    }
}
