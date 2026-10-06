use reconciliation::{InspectError, WorkspaceInspector, WorkspaceObservation};

pub struct PlatformWorkspaceInspector;

#[cfg(unix)]
impl WorkspaceInspector for PlatformWorkspaceInspector {
    fn inspect(&self, user: &str, home: &str) -> Result<WorkspaceObservation, InspectError> {
        use std::os::unix::fs::MetadataExt;

        let user_uid = user_uid(user);
        let metadata = std::fs::metadata(home).ok();

        Ok(WorkspaceObservation {
            user_exists: user_uid.is_some(),
            user_uid,
            home_exists: metadata.is_some(),
            home_uid: metadata.as_ref().map(|meta| meta.uid()),
            home_mode: metadata.as_ref().map(|meta| meta.mode()),
        })
    }
}

#[cfg(not(unix))]
impl WorkspaceInspector for PlatformWorkspaceInspector {
    fn inspect(&self, _user: &str, _home: &str) -> Result<WorkspaceObservation, InspectError> {
        Err(InspectError::Unsupported)
    }
}

#[cfg(unix)]
fn user_uid(user: &str) -> Option<u32> {
    use std::process::{Command, Stdio};

    let output = Command::new("id")
        .arg("-u")
        .arg(user)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8_lossy(&output.stdout).trim().parse().ok()
}
