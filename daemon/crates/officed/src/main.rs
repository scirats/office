use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::time::Duration;

use officed::Daemon;
use officed::adapters::{PlatformWorkspaceInspector, SqliteStore};

const DEFAULT_SOCKET: &str = "/run/office/office.sock";
const DEFAULT_DB: &str = "/var/lib/office/state.sqlite";
const DEFAULT_ADDR: &str = "127.0.0.1:7717";

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let socket_path = std::env::var_os("OFFICE_SOCKET")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_SOCKET));

    let database = std::env::var("OFFICE_DB").unwrap_or_else(|_| DEFAULT_DB.to_owned());

    let tcp_addr = match std::env::var("OFFICE_ADDR") {
        Ok(value) if value.is_empty() || value == "none" => None,
        Ok(value) => Some(value.parse::<SocketAddr>().map_err(|error| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("invalid OFFICE_ADDR: {error}"),
            )
        })?),
        Err(_) => Some(DEFAULT_ADDR.parse().expect("default address is valid")),
    };

    let store = if database == ":memory:" {
        SqliteStore::open_in_memory()
    } else {
        SqliteStore::open(Path::new(&database))
    }
    .map_err(|error| {
        std::io::Error::other(format!(
            "could not open the state database '{database}': {error}"
        ))
    })?;
    let agent_store = store.clone();

    match tcp_addr {
        Some(addr) => eprintln!(
            "office: listening on {} and tcp {addr}, state {database}",
            socket_path.display()
        ),
        None => eprintln!(
            "office: listening on {}, state {database}",
            socket_path.display()
        ),
    }

    let mut daemon = Daemon::new(
        Box::new(store),
        Box::new(agent_store),
        Box::new(PlatformWorkspaceInspector),
    );
    if let Some(seconds) = std::env::var("OFFICE_RECONCILE_SECS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
    {
        daemon = daemon.with_reconcile_interval(Duration::from_secs(seconds));
    }
    daemon.reconcile();
    daemon.serve(socket_path, tcp_addr).await
}
