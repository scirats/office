use std::process::{Command, Stdio};

use workspace::{ProvisionError, Provisioned, UserProvisioner, home_path};

pub struct PlatformUserProvisioner;

impl UserProvisioner for PlatformUserProvisioner {
    fn provision(&self, slug: &str) -> Result<Provisioned, ProvisionError> {
        if !cfg!(target_os = "linux") {
            return Err(ProvisionError::Unsupported);
        }

        let home = home_path(slug);
        run("groupadd", &["--force", slug], "groupadd", slug)?;

        if !user_exists(slug)? {
            run(
                "useradd",
                &["--create-home", "--gid", slug, slug],
                "useradd",
                slug,
            )?;
        }

        run(
            "install",
            &["-d", "-m", "0700", "-o", slug, "-g", slug, home.as_str()],
            "install",
            &home,
        )?;

        Ok(Provisioned {
            user: slug.to_owned(),
            home,
        })
    }
}

fn user_exists(slug: &str) -> Result<bool, ProvisionError> {
    let status = Command::new("id")
        .arg("-u")
        .arg(slug)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|error| command_error("id", slug, error.to_string()))?;
    Ok(status.success())
}

fn run(program: &str, args: &[&str], action: &str, target: &str) -> Result<(), ProvisionError> {
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|error| command_error(action, target, error.to_string()))?;

    if output.status.success() {
        return Ok(());
    }

    let message = String::from_utf8_lossy(&output.stderr).trim().to_owned();
    Err(command_error(action, target, message))
}

fn command_error(action: &str, target: &str, message: String) -> ProvisionError {
    ProvisionError::Command {
        action: action.to_owned(),
        target: target.to_owned(),
        message,
    }
}
