use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::time::Duration;

use officed::Daemon;
use officed::adapters::{
    FileModelCatalog, FileSecretStore, PlatformUserProvisioner, PlatformWorkspaceInspector,
    ReqwestHttpClient, SqliteStore,
};

use model::ModelCatalog;
use secret::SecretStore;
use tracing_subscriber::EnvFilter;

const DEFAULT_SOCKET: &str = "/run/office/office.sock";
const DEFAULT_DB: &str = "/var/lib/office/state.sqlite";
const DEFAULT_ADDR: &str = "127.0.0.1:7717";

#[tokio::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .compact()
        .init();

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
    let workspace_store = store.clone();
    let project_store = store;

    tracing::info!(
        socket = %socket_path.display(),
        tcp = ?tcp_addr,
        database = %database,
        "office listening"
    );

    let models_path = FileModelCatalog::path_from_env();
    let models: Box<dyn ModelCatalog> = if models_path.exists() {
        match FileModelCatalog::load(&models_path) {
            Ok(catalog) => Box::new(catalog),
            Err(error) => {
                return Err(std::io::Error::other(format!(
                    "invalid models config '{}': {error}",
                    models_path.display()
                )));
            }
        }
    } else {
        tracing::warn!(
            path = %models_path.display(),
            "models config not found; model catalog is empty"
        );
        Box::new(FileModelCatalog::empty())
    };

    let secrets_path = FileSecretStore::path_from_env();
    if !secrets_path.exists() {
        tracing::warn!(
            path = %secrets_path.display(),
            "secrets file not found; it will be created on first set"
        );
    }
    let secrets: Box<dyn SecretStore> = match FileSecretStore::open(&secrets_path) {
        Ok(store) => Box::new(store),
        Err(error) => {
            return Err(std::io::Error::other(format!(
                "invalid secrets file '{}': {error}",
                secrets_path.display()
            )));
        }
    };

    let mut daemon = Daemon::with_adapters(
        Box::new(PlatformUserProvisioner),
        Box::new(workspace_store),
        Box::new(agent_store),
        Box::new(project_store),
        Box::new(PlatformWorkspaceInspector),
        models,
        secrets,
        std::sync::Arc::new(ReqwestHttpClient::new()),
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
