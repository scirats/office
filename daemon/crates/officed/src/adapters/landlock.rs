use std::path::{Path, PathBuf};

use landlock::{
    ABI, Access, AccessFs, Ruleset, RulesetAttr, RulesetCreated, RulesetCreatedAttr,
    path_beneath_rules,
};

pub struct LandlockRuleset {
    created: RulesetCreated,
}

#[derive(Debug, thiserror::Error)]
pub enum LandlockError {
    #[error("could not build the Landlock ruleset: {0}")]
    Ruleset(String),
}

pub fn prepare(writable: &[PathBuf]) -> Result<LandlockRuleset, LandlockError> {
    let abi = ABI::V1;
    let created = Ruleset::default()
        .handle_access(AccessFs::from_all(abi))
        .map_err(fail)?
        .create()
        .map_err(fail)?
        .add_rules(path_beneath_rules(
            [Path::new("/")],
            AccessFs::from_read(abi),
        ))
        .map_err(fail)?;

    let writable: Vec<&Path> = writable
        .iter()
        .map(PathBuf::as_path)
        .filter(|path| path.exists())
        .collect();
    let created = if writable.is_empty() {
        created
    } else {
        created
            .add_rules(path_beneath_rules(writable, AccessFs::from_all(abi)))
            .map_err(fail)?
    };

    Ok(LandlockRuleset { created })
}

impl LandlockRuleset {
    pub fn restrict_self(self) -> std::io::Result<()> {
        self.created
            .restrict_self()
            .map(|_| ())
            .map_err(|error| std::io::Error::other(error.to_string()))
    }
}

fn fail(error: impl std::fmt::Display) -> LandlockError {
    LandlockError::Ruleset(error.to_string())
}

#[cfg(test)]
mod tests {
    use std::os::unix::process::CommandExt;
    use std::process::Command;

    use super::*;

    fn prepare_or_skip(writable: &Path) -> Option<LandlockRuleset> {
        match prepare(&[writable.to_path_buf()]) {
            Ok(ruleset) => Some(ruleset),
            Err(_) => {
                eprintln!("skipping: Landlock is not available");
                None
            }
        }
    }

    fn run(ruleset: LandlockRuleset, script: &str) -> std::process::Output {
        let mut command = Command::new("/bin/sh");
        command.arg("-c").arg(script);
        let mut ruleset = Some(ruleset);

        unsafe {
            command.pre_exec(move || match ruleset.take() {
                Some(ruleset) => ruleset.restrict_self(),
                None => Ok(()),
            });
        }
        command.output().expect("spawns")
    }

    #[test]
    fn allows_writes_inside_and_denies_outside() {
        let writable = std::env::temp_dir().join(format!("office-landlock-{}", std::process::id()));
        std::fs::create_dir_all(&writable).expect("writable dir");
        let Some(ruleset) = prepare_or_skip(&writable) else {
            return;
        };

        let allowed = run(
            ruleset,
            &format!(
                "echo ok > {}/inside && cat {}/inside",
                writable.display(),
                writable.display()
            ),
        );
        assert!(
            allowed.status.success(),
            "stderr: {}",
            String::from_utf8_lossy(&allowed.stderr)
        );
        assert!(String::from_utf8_lossy(&allowed.stdout).contains("ok"));

        let Some(ruleset) = prepare_or_skip(&writable) else {
            return;
        };
        let denied = run(
            ruleset,
            "echo no > /etc/office-landlock-denied 2>/dev/null; echo $?",
        );
        assert!(
            !String::from_utf8_lossy(&denied.stdout)
                .trim()
                .starts_with('0'),
            "write outside should have failed"
        );

        let _ = std::fs::remove_dir_all(&writable);
    }
}
