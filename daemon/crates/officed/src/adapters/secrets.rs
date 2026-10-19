use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use secret::{Secret, SecretError, SecretStore};
use serde::{Deserialize, Serialize};

const DEFAULT_CONFIG: &str = "/etc/office/secrets.toml";
const ENV_VAR: &str = "OFFICE_SECRETS";

#[derive(Debug, thiserror::Error)]
pub enum SecretConfigError {
    #[error("could not read secrets file '{path}': {source}")]
    Read {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("could not parse secrets file '{path}': {source}")]
    Parse {
        path: String,
        #[source]
        source: toml::de::Error,
    },
    #[error("secrets file '{path}' must be mode 0600, found {mode:04o}")]
    InsecurePermissions { path: String, mode: u32 },
}

#[derive(Debug, Deserialize)]
struct ConfigFile {
    #[serde(default)]
    secrets: BTreeMap<String, String>,
}

#[derive(Serialize)]
struct WriterFile<'a> {
    secrets: BTreeMap<&'a str, &'a str>,
}

#[derive(Debug)]
pub struct FileSecretStore {
    path: PathBuf,
    secrets: Mutex<BTreeMap<String, Secret>>,
}

impl FileSecretStore {
    pub fn empty() -> Self {
        Self {
            path: Self::path_from_env(),
            secrets: Mutex::new(BTreeMap::new()),
        }
    }

    pub fn path_from_env() -> PathBuf {
        std::env::var_os(ENV_VAR)
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(DEFAULT_CONFIG))
    }

    pub fn open(path: &Path) -> Result<Self, SecretConfigError> {
        if path.exists() {
            Self::load(path)
        } else {
            Ok(Self {
                path: path.to_owned(),
                secrets: Mutex::new(BTreeMap::new()),
            })
        }
    }

    pub fn load(path: &Path) -> Result<Self, SecretConfigError> {
        enforce_mode(path)?;
        let text = std::fs::read_to_string(path).map_err(|source| SecretConfigError::Read {
            path: path.display().to_string(),
            source,
        })?;
        let secrets = parse(&text, &path.display().to_string())?;
        Ok(Self {
            path: path.to_owned(),
            secrets: Mutex::new(secrets),
        })
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, BTreeMap<String, Secret>>, SecretError> {
        self.secrets
            .lock()
            .map_err(|_| SecretError::Persist("store is poisoned".to_owned()))
    }

    fn persist(&self, secrets: &BTreeMap<String, Secret>) -> Result<(), SecretError> {
        let pairs: BTreeMap<&str, &str> = secrets
            .iter()
            .map(|(key, value)| (key.as_str(), value.expose()))
            .collect();
        let data = toml::to_string(&WriterFile { secrets: pairs })
            .map_err(|error| SecretError::Persist(error.to_string()))?;

        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|error| SecretError::Persist(error.to_string()))?;
        }
        let temp = self.path.with_extension("tmp");
        write_private(&temp, data.as_bytes())?;
        std::fs::rename(&temp, &self.path).map_err(|error| SecretError::Persist(error.to_string()))
    }
}

impl SecretStore for FileSecretStore {
    fn resolve(&self, reference: &str) -> Option<Secret> {
        self.lock().ok()?.get(reference).cloned()
    }

    fn references(&self) -> Vec<String> {
        self.lock()
            .map(|secrets| secrets.keys().cloned().collect())
            .unwrap_or_default()
    }

    fn set(&self, reference: &str, value: &str) -> Result<(), SecretError> {
        let value = value.trim();
        if value.is_empty() {
            return Err(SecretError::Persist("secret value is empty".to_owned()));
        }
        let mut secrets = self.lock()?;
        secrets.insert(reference.to_owned(), Secret::new(value));
        self.persist(&secrets)
    }

    fn reload(&self) -> Result<(), SecretError> {
        let loaded = if self.path.exists() {
            FileSecretStore::load(&self.path)
                .map_err(|error| SecretError::Reload(error.to_string()))?
        } else {
            FileSecretStore {
                path: self.path.clone(),
                secrets: Mutex::new(BTreeMap::new()),
            }
        };
        let mut guard = self.lock()?;
        *guard = loaded
            .secrets
            .into_inner()
            .map_err(|_| SecretError::Reload("store is poisoned".to_owned()))?;
        Ok(())
    }
}

fn parse(text: &str, path: &str) -> Result<BTreeMap<String, Secret>, SecretConfigError> {
    let file: ConfigFile = toml::from_str(text).map_err(|source| SecretConfigError::Parse {
        path: path.to_owned(),
        source,
    })?;
    Ok(file
        .secrets
        .into_iter()
        .map(|(key, value)| (key, Secret::new(value)))
        .collect())
}

fn write_private(path: &Path, contents: &[u8]) -> Result<(), SecretError> {
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options
        .open(path)
        .map_err(|error| SecretError::Persist(error.to_string()))?;
    file.write_all(contents)
        .map_err(|error| SecretError::Persist(error.to_string()))?;
    drop(file);
    set_private_mode(path)
}

fn set_private_mode(path: &Path) -> Result<(), SecretError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))
            .map_err(|error| SecretError::Persist(error.to_string()))?;
    }
    #[cfg(not(unix))]
    {
        let _ = path;
    }
    Ok(())
}

fn enforce_mode(path: &Path) -> Result<(), SecretConfigError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let metadata = std::fs::metadata(path).map_err(|source| SecretConfigError::Read {
            path: path.display().to_string(),
            source,
        })?;
        let mode = metadata.permissions().mode() & 0o777;
        if mode & 0o077 != 0 {
            return Err(SecretConfigError::InsecurePermissions {
                path: path.display().to_string(),
                mode,
            });
        }
    }
    #[cfg(not(unix))]
    {
        let _ = path;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_path(name: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!("office-secret-{}-{name}.toml", std::process::id()));
        path
    }

    const SAMPLE: &str = r#"
[secrets]
"provider/anthropic" = "sk-ant-123"
"provider/openai" = "sk-openai-456"
"#;

    fn write_with_mode(path: &Path, contents: &str, mode: u32) {
        std::fs::write(path, contents).expect("writes");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode)).expect("chmod");
        }
    }

    #[test]
    fn resolves_configured_references() {
        let path = temp_path("resolve");
        write_with_mode(&path, SAMPLE, 0o600);
        let store = FileSecretStore::load(&path).expect("loads");
        assert_eq!(
            store
                .resolve("provider/anthropic")
                .expect("present")
                .expose(),
            "sk-ant-123"
        );
        assert!(store.resolve("provider/ghost").is_none());
        assert_eq!(
            store.references(),
            vec!["provider/anthropic", "provider/openai"]
        );
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn sets_and_persists_with_0600() {
        let path = temp_path("set");
        let _ = std::fs::remove_file(&path);

        let store = FileSecretStore::open(&path).expect("opens empty");
        store.set("provider/anthropic", "sk-ant-new").expect("sets");

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&path)
                .expect("metadata")
                .permissions()
                .mode()
                & 0o777;
            assert_eq!(mode, 0o600);
        }

        let reopened = FileSecretStore::open(&path).expect("reopens");
        assert_eq!(
            reopened
                .resolve("provider/anthropic")
                .expect("present")
                .expose(),
            "sk-ant-new"
        );
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(path.with_extension("tmp"));
    }

    #[test]
    fn reload_reflects_external_edits() {
        let path = temp_path("reload");
        write_with_mode(&path, "[secrets]\n\"provider/a\" = \"one\"\n", 0o600);
        let store = FileSecretStore::load(&path).expect("loads");
        assert_eq!(
            store.resolve("provider/a").expect("present").expose(),
            "one"
        );

        write_with_mode(&path, "[secrets]\n\"provider/a\" = \"two\"\n", 0o600);
        store.reload().expect("reloads");
        assert_eq!(
            store.resolve("provider/a").expect("present").expose(),
            "two"
        );

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn rejects_empty_value() {
        let path = temp_path("empty");
        let _ = std::fs::remove_file(&path);
        let store = FileSecretStore::open(&path).expect("opens");
        assert!(store.set("provider/x", "   ").is_err());
    }

    #[test]
    fn rejects_insecure_permissions() {
        let path = temp_path("insecure");
        write_with_mode(&path, SAMPLE, 0o644);
        let error = FileSecretStore::load(&path).unwrap_err();
        assert!(matches!(
            error,
            SecretConfigError::InsecurePermissions { mode, .. } if mode == 0o644
        ));
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn reports_unreadable_file() {
        let error = FileSecretStore::load(Path::new("/nonexistent/secrets.toml")).unwrap_err();
        assert!(matches!(error, SecretConfigError::Read { .. }));
    }
}
