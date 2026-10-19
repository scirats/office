use std::path::Path;
use std::time::Duration;

use officed::adapters::BwrapSandbox;
use sandbox::{Mount, Sandbox, SandboxSpec};

fn bwrap_available() -> bool {
    std::process::Command::new("bwrap")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

#[tokio::test]
async fn runs_a_command_inside_the_jail() {
    if !bwrap_available() {
        eprintln!("skipping: bwrap is not available");
        return;
    }

    let workspace = std::env::temp_dir().join(format!("office-sandbox-{}", std::process::id()));
    std::fs::create_dir_all(&workspace).expect("workspace");

    let mut mounts: Vec<Mount> = ["/usr", "/bin", "/lib", "/lib64"]
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

    let spec = SandboxSpec {
        execution_id: format!("test-{}", std::process::id()),
        program: "/bin/echo".to_owned(),
        args: vec!["sandbox-ok".to_owned()],
        workdir: "/workspace".to_owned(),
        env: Vec::new(),
        mounts,
        network: false,
        limits: Default::default(),
        timeout: Duration::from_secs(10),
    };

    let output = BwrapSandbox::new(None)
        .run(spec)
        .await
        .expect("sandbox runs");
    assert_eq!(output.exit_code, Some(0));
    assert!(
        output.stdout.contains("sandbox-ok"),
        "unexpected stdout: {:?}",
        output.stdout
    );

    let _ = std::fs::remove_dir_all(&workspace);
}
