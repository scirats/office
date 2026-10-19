use std::path::PathBuf;
use std::time::Duration;

const DEFAULT_EXECUTIONS_DIR: &str = "/run/office/executions";
const DEFAULT_MAX_STEPS: u32 = 8;
const DEFAULT_TOOL_TIMEOUT_SECS: u64 = 30;

#[derive(Debug, Clone)]
pub struct ExecutionPolicy {
    pub max_steps: u32,
    pub tool_timeout: Duration,
    pub allow_network: bool,
    pub executions_dir: PathBuf,

    pub cgroup_root: Option<PathBuf>,
    pub memory_max_bytes: Option<u64>,
    pub pids_max: Option<u32>,

    pub landlock: bool,
}

impl ExecutionPolicy {
    pub fn from_env() -> Self {
        Self {
            max_steps: env_u32("OFFICE_MAX_STEPS", DEFAULT_MAX_STEPS),
            tool_timeout: Duration::from_secs(env_u32(
                "OFFICE_TOOL_TIMEOUT_SECS",
                DEFAULT_TOOL_TIMEOUT_SECS as u32,
            ) as u64),
            allow_network: env_flag("OFFICE_ALLOW_NETWORK"),
            executions_dir: std::env::var_os("OFFICE_EXECUTIONS_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from(DEFAULT_EXECUTIONS_DIR)),
            cgroup_root: cgroup_root(),
            memory_max_bytes: std::env::var("OFFICE_MEMORY_MAX_MB")
                .ok()
                .and_then(|value| value.parse::<u64>().ok())
                .map(|mebibytes| mebibytes * 1024 * 1024),
            pids_max: std::env::var("OFFICE_PIDS_MAX")
                .ok()
                .and_then(|value| value.parse().ok()),
            landlock: std::env::var("OFFICE_LANDLOCK")
                .map(|value| value != "0" && !value.eq_ignore_ascii_case("false"))
                .unwrap_or(true),
        }
    }

    pub fn execution_dir(&self, execution_id: &str) -> PathBuf {
        self.executions_dir.join(sanitize(execution_id))
    }
}

impl Default for ExecutionPolicy {
    fn default() -> Self {
        Self::from_env()
    }
}

fn cgroup_root() -> Option<PathBuf> {
    match std::env::var_os("OFFICE_CGROUP_ROOT") {
        Some(value) if value == "none" => None,
        Some(value) => Some(PathBuf::from(value)),
        None => Some(PathBuf::from("/sys/fs/cgroup")),
    }
}

fn env_u32(key: &str, fallback: u32) -> u32 {
    std::env::var(key)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(fallback)
}

fn env_flag(key: &str) -> bool {
    std::env::var(key)
        .map(|value| value == "1" || value.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

fn sanitize(value: &str) -> String {
    value
        .chars()
        .filter(|character| {
            character.is_ascii_alphanumeric() || *character == '_' || *character == '-'
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitizes_execution_ids() {
        let policy = ExecutionPolicy {
            executions_dir: PathBuf::from("/run/office/executions"),
            ..ExecutionPolicy::from_env()
        };
        assert_eq!(
            policy.execution_dir("exec_1"),
            PathBuf::from("/run/office/executions/exec_1")
        );
        assert_eq!(
            policy.execution_dir("../../etc"),
            PathBuf::from("/run/office/executions/etc")
        );
    }
}
