use std::time::Duration;

use sandbox::{SandboxError, SandboxOutput};

use super::landlock::LandlockRuleset;
use super::process::{CgroupConfig, capture};

#[allow(clippy::too_many_arguments)]
pub async fn run(
    program: &str,
    args: &[String],
    workdir: &str,
    env: &[(String, String)],
    uid: Option<u32>,
    gid: Option<u32>,
    timeout: Duration,
    cgroup: Option<CgroupConfig>,
    landlock: Option<LandlockRuleset>,
) -> Result<SandboxOutput, SandboxError> {
    let mut command = tokio::process::Command::new(program);
    command.args(args).current_dir(workdir);
    for (key, value) in env {
        command.env(key, value);
    }
    #[cfg(unix)]
    {
        if let Some(uid) = uid {
            command.uid(uid);
        }
        if let Some(gid) = gid {
            command.gid(gid);
        }
    }
    #[cfg(not(unix))]
    {
        let _ = (uid, gid);
    }

    if let Some(ruleset) = landlock {
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            let mut ruleset = Some(ruleset);

            unsafe {
                command.as_std_mut().pre_exec(move || match ruleset.take() {
                    Some(ruleset) => ruleset.restrict_self(),
                    None => Ok(()),
                });
            }
        }
        #[cfg(not(unix))]
        {
            let _ = ruleset;
        }
    }

    capture(command, timeout, cgroup).await
}
