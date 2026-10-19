use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ResourceLimits {
    pub memory_max_bytes: Option<u64>,
    pub pids_max: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mount {
    pub source: String,
    pub target: String,
    pub read_only: bool,
}

#[derive(Debug, Clone)]
pub struct SandboxSpec {
    pub execution_id: String,
    pub program: String,
    pub args: Vec<String>,
    pub workdir: String,
    pub env: Vec<(String, String)>,
    pub mounts: Vec<Mount>,

    pub network: bool,
    pub limits: ResourceLimits,
    pub timeout: Duration,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SandboxOutput {
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

impl SandboxOutput {
    pub fn report(&self) -> String {
        let code = self
            .exit_code
            .map(|value| value.to_string())
            .unwrap_or_else(|| "signal".to_owned());
        let mut report = format!("exit {code}\n");
        report.push_str(&self.stdout);
        if !self.stderr.is_empty() {
            report.push_str("\n[stderr]\n");
            report.push_str(&self.stderr);
        }
        report
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SandboxError {
    #[error("sandboxed command failed: {0}")]
    Failed(String),
    #[error("sandboxed command timed out")]
    Timeout,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_exit_and_streams() {
        let output = SandboxOutput {
            exit_code: Some(0),
            stdout: "ok".to_owned(),
            stderr: "warn".to_owned(),
        };
        assert_eq!(output.report(), "exit 0\nok\n[stderr]\nwarn");
    }
}
