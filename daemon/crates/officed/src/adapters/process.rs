use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;

use sandbox::{SandboxError, SandboxOutput};
use tokio::io::AsyncReadExt;

#[derive(Debug, Clone)]
pub struct CgroupConfig {
    pub root: PathBuf,
    pub name: String,
    pub memory_max_bytes: Option<u64>,
    pub pids_max: Option<u32>,
}

struct Cgroup {
    dir: PathBuf,
}

impl Cgroup {
    fn create(config: &CgroupConfig) -> Option<Self> {
        let base = config.root.join("office");
        let dir = base.join("executions").join(&config.name);
        std::fs::create_dir_all(&dir).ok()?;

        enable_controllers(&config.root, &base);
        if let Some(bytes) = config.memory_max_bytes {
            let _ = std::fs::write(dir.join("memory.max"), bytes.to_string());
        }
        if let Some(pids) = config.pids_max {
            let _ = std::fs::write(dir.join("pids.max"), pids.to_string());
        }
        Some(Self { dir })
    }

    fn attach(&self, pid: u32) {
        let _ = std::fs::write(self.dir.join("cgroup.procs"), pid.to_string());
    }

    fn kill(&self) {
        let _ = std::fs::write(self.dir.join("cgroup.kill"), "1");
    }

    fn remove(&self) {
        let _ = std::fs::remove_dir(&self.dir);
    }
}

fn enable_controllers(root: &std::path::Path, base: &std::path::Path) {
    let executions = base.join("executions");
    let _ = std::fs::create_dir_all(&executions);

    let _ = std::fs::write(root.join("cgroup.subtree_control"), "+memory +pids");
    let _ = std::fs::write(base.join("cgroup.subtree_control"), "+memory +pids");
    let _ = std::fs::write(executions.join("cgroup.subtree_control"), "+memory +pids");
}

pub async fn capture(
    mut command: tokio::process::Command,
    timeout: Duration,
    cgroup: Option<CgroupConfig>,
) -> Result<SandboxOutput, SandboxError> {
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.as_std_mut().process_group(0);
    }
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let cgroup = cgroup.and_then(|config| Cgroup::create(&config));

    let mut child = command
        .spawn()
        .map_err(|error| SandboxError::Failed(error.to_string()))?;
    let pid = child.id();
    if let (Some(cgroup), Some(pid)) = (&cgroup, pid) {
        cgroup.attach(pid);
    }

    let mut stdout = child.stdout.take();
    let mut stderr = child.stderr.take();
    let stdout_task = tokio::spawn(async move {
        let mut buffer = Vec::new();
        if let Some(stream) = stdout.as_mut() {
            let _ = stream.read_to_end(&mut buffer).await;
        }
        buffer
    });
    let stderr_task = tokio::spawn(async move {
        let mut buffer = Vec::new();
        if let Some(stream) = stderr.as_mut() {
            let _ = stream.read_to_end(&mut buffer).await;
        }
        buffer
    });

    let result = match tokio::time::timeout(timeout, child.wait()).await {
        Ok(Ok(status)) => {
            let stdout = stdout_task.await.unwrap_or_default();
            let stderr = stderr_task.await.unwrap_or_default();
            Ok(SandboxOutput {
                exit_code: status.code(),
                stdout: String::from_utf8_lossy(&stdout).into_owned(),
                stderr: String::from_utf8_lossy(&stderr).into_owned(),
            })
        }
        Ok(Err(error)) => {
            stdout_task.abort();
            stderr_task.abort();
            kill(&cgroup, pid);
            let _ = child.wait().await;
            Err(SandboxError::Failed(error.to_string()))
        }
        Err(_) => {
            stdout_task.abort();
            stderr_task.abort();
            kill(&cgroup, pid);
            let _ = child.wait().await;
            Err(SandboxError::Timeout)
        }
    };

    if let Some(cgroup) = &cgroup {
        cgroup.remove();
    }
    result
}

fn kill(cgroup: &Option<Cgroup>, pid: Option<u32>) {
    if let Some(cgroup) = cgroup {
        cgroup.kill();
    }
    kill_group(pid);
}

#[cfg(unix)]
fn kill_group(pid: Option<u32>) {
    if let Some(pid) = pid {
        use nix::sys::signal::{Signal, killpg};
        use nix::unistd::Pid;
        let _ = killpg(Pid::from_raw(pid as i32), Signal::SIGKILL);
    }
}

#[cfg(not(unix))]
fn kill_group(_pid: Option<u32>) {}
