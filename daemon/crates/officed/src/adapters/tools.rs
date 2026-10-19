use std::path::{Path, PathBuf};
use std::sync::Arc;

use futures_util::future::BoxFuture;
use runner::{RunContext, RunScope, ToolCall, ToolError, ToolRunner};
use sandbox::{Mount, ResourceLimits, Sandbox, SandboxSpec};

use super::host;
use super::landlock::{LandlockRuleset, prepare as prepare_landlock};
use super::process::CgroupConfig;
use crate::policy::ExecutionPolicy;

const READONLY_ROOTS: [&str; 6] = ["/usr", "/bin", "/sbin", "/lib", "/lib64", "/etc"];

pub struct ToolExecutor {
    sandbox: Arc<dyn Sandbox>,
    policy: ExecutionPolicy,
}

impl ToolExecutor {
    pub fn new(sandbox: Arc<dyn Sandbox>, policy: ExecutionPolicy) -> Self {
        Self { sandbox, policy }
    }
}

impl ToolRunner for ToolExecutor {
    fn run(
        &self,
        call: ToolCall,
        context: RunContext,
    ) -> BoxFuture<'static, Result<String, ToolError>> {
        let sandbox = self.sandbox.clone();
        let policy = self.policy.clone();
        Box::pin(async move {
            let fail = |message: String| ToolError::Failed {
                name: call.name.clone(),
                message,
            };
            let action = toolbox::action(&call.name, &call.arguments)
                .map_err(|error| fail(error.to_string()))?;

            let output = match &context.scope {
                RunScope::Agent => {
                    let workspace = agent_workspace(&policy, &context.execution_id)
                        .map_err(|error| fail(error.to_string()))?;
                    let spec = SandboxSpec {
                        execution_id: context.execution_id.clone(),
                        program: action.program,
                        args: action.args,
                        workdir: "/workspace".to_owned(),
                        env: vec![
                            ("HOME".to_owned(), "/workspace".to_owned()),
                            ("PATH".to_owned(), "/usr/bin:/bin".to_owned()),
                        ],
                        mounts: agent_mounts(&workspace),
                        network: policy.allow_network,
                        limits: ResourceLimits {
                            memory_max_bytes: policy.memory_max_bytes,
                            pids_max: policy.pids_max,
                        },
                        timeout: policy.tool_timeout,
                    };
                    sandbox.run(spec).await
                }
                RunScope::Instance {
                    workspace,
                    home,
                    project,
                } => {
                    let (uid, gid) = user_ids(workspace).unzip();
                    let env = vec![
                        ("HOME".to_owned(), home.clone()),
                        ("PATH".to_owned(), "/usr/bin:/bin".to_owned()),
                    ];
                    let cgroup = policy.cgroup_root.as_ref().map(|root| CgroupConfig {
                        root: root.clone(),
                        name: context.execution_id.clone(),
                        memory_max_bytes: policy.memory_max_bytes,
                        pids_max: policy.pids_max,
                    });
                    let landlock = landlock_for(&policy, project, home);
                    host::run(
                        &action.program,
                        &action.args,
                        project,
                        &env,
                        uid,
                        gid,
                        policy.tool_timeout,
                        cgroup,
                        landlock,
                    )
                    .await
                }
            }
            .map_err(|error| fail(error.to_string()))?;

            Ok(output.report())
        })
    }
}

fn agent_workspace(
    policy: &ExecutionPolicy,
    execution_id: &str,
) -> Result<PathBuf, std::io::Error> {
    let workspace = policy.execution_dir(execution_id).join("workspace");
    std::fs::create_dir_all(&workspace)?;
    Ok(workspace)
}

fn landlock_for(policy: &ExecutionPolicy, project: &str, home: &str) -> Option<LandlockRuleset> {
    if !policy.landlock {
        return None;
    }
    let writable = vec![
        PathBuf::from(project),
        PathBuf::from(home),
        PathBuf::from("/tmp"),
        policy.executions_dir.clone(),
    ];
    match prepare_landlock(&writable) {
        Ok(ruleset) => Some(ruleset),
        Err(error) => {
            tracing::warn!(
                target: "office.execution",
                error = %error,
                "Landlock is not available; running without it"
            );
            None
        }
    }
}

fn agent_mounts(workspace: &Path) -> Vec<Mount> {
    let mut mounts: Vec<Mount> = READONLY_ROOTS
        .iter()
        .filter(|root| Path::new(root).exists())
        .map(|root| Mount {
            source: (*root).to_owned(),
            target: (*root).to_owned(),
            read_only: true,
        })
        .collect();
    mounts.push(Mount {
        source: workspace.to_string_lossy().into_owned(),
        target: "/workspace".to_owned(),
        read_only: false,
    });
    mounts
}

fn user_ids(user: &str) -> Option<(u32, u32)> {
    let contents = std::fs::read_to_string("/etc/passwd").ok()?;
    for line in contents.lines() {
        let mut fields = line.split(':');
        if fields.next() != Some(user) {
            continue;
        }
        let _password = fields.next();
        let uid = fields.next()?.parse().ok()?;
        let gid = fields.next()?.parse().ok()?;
        return Some((uid, gid));
    }
    None
}
