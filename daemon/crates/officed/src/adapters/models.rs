use std::path::{Path, PathBuf};

use model::{Capabilities, Model, ModelCatalog, ModelId, Provider, ProviderId, ProviderKind};
use serde::Deserialize;

const DEFAULT_CONFIG: &str = "/etc/office/models.toml";
const ENV_VAR: &str = "OFFICE_MODELS";

#[derive(Debug, thiserror::Error)]
pub enum ModelConfigError {
    #[error("could not read models config '{path}': {source}")]
    Read {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("could not parse models config '{path}': {source}")]
    Parse {
        path: String,
        #[source]
        source: toml::de::Error,
    },
    #[error("duplicate provider id '{0}'")]
    DuplicateProvider(String),
    #[error("duplicate model id '{0}'")]
    DuplicateModel(String),
    #[error("model '{model}' references unknown provider '{provider}'")]
    UnknownProvider { model: String, provider: String },
}

#[derive(Debug, Deserialize)]
struct ConfigFile {
    #[serde(default)]
    provider: Vec<ProviderEntry>,
    #[serde(default)]
    model: Vec<ModelEntry>,
}

#[derive(Debug, Deserialize)]
struct ProviderEntry {
    id: String,
    kind: String,
    endpoint: String,
    credential_ref: String,
}

#[derive(Debug, Deserialize)]
struct ModelEntry {
    id: String,
    provider: String,
    api_name: String,
    display_name: String,
    context_window: u32,
    max_output_tokens: u32,
    #[serde(default)]
    capabilities: CapabilitiesEntry,
}

#[derive(Debug, Default, Deserialize)]
struct CapabilitiesEntry {
    #[serde(default)]
    tool_calling: bool,
    #[serde(default)]
    vision: bool,
    #[serde(default)]
    reasoning: bool,
}

#[derive(Debug)]
pub struct FileModelCatalog {
    providers: Vec<Provider>,
    models: Vec<Model>,
}

impl FileModelCatalog {
    pub fn empty() -> Self {
        Self {
            providers: Vec::new(),
            models: Vec::new(),
        }
    }

    pub fn path_from_env() -> PathBuf {
        std::env::var_os(ENV_VAR)
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(DEFAULT_CONFIG))
    }

    pub fn load(path: &Path) -> Result<Self, ModelConfigError> {
        let text = std::fs::read_to_string(path).map_err(|source| ModelConfigError::Read {
            path: path.display().to_string(),
            source,
        })?;
        Self::parse(&text, &path.display().to_string())
    }

    fn parse(text: &str, path: &str) -> Result<Self, ModelConfigError> {
        let file: ConfigFile = toml::from_str(text).map_err(|source| ModelConfigError::Parse {
            path: path.to_owned(),
            source,
        })?;

        let mut providers: Vec<Provider> = Vec::with_capacity(file.provider.len());
        for entry in file.provider {
            if providers.iter().any(|item| item.id.as_str() == entry.id) {
                return Err(ModelConfigError::DuplicateProvider(entry.id));
            }
            providers.push(Provider {
                id: ProviderId(entry.id),
                kind: ProviderKind::parse(&entry.kind),
                endpoint: entry.endpoint,
                credential_ref: entry.credential_ref,
            });
        }

        let mut models: Vec<Model> = Vec::with_capacity(file.model.len());
        for entry in file.model {
            if models.iter().any(|item| item.id.as_str() == entry.id) {
                return Err(ModelConfigError::DuplicateModel(entry.id));
            }
            if !providers
                .iter()
                .any(|item| item.id.as_str() == entry.provider)
            {
                return Err(ModelConfigError::UnknownProvider {
                    model: entry.id,
                    provider: entry.provider,
                });
            }
            models.push(Model {
                id: ModelId(entry.id),
                provider: ProviderId(entry.provider),
                api_name: entry.api_name,
                display_name: entry.display_name,
                context_window: entry.context_window,
                max_output_tokens: entry.max_output_tokens,
                capabilities: Capabilities {
                    tool_calling: entry.capabilities.tool_calling,
                    vision: entry.capabilities.vision,
                    reasoning: entry.capabilities.reasoning,
                },
            });
        }

        Ok(Self { providers, models })
    }
}

impl ModelCatalog for FileModelCatalog {
    fn list(&self) -> Vec<Model> {
        self.models.clone()
    }

    fn get(&self, id: &str) -> Option<Model> {
        self.models
            .iter()
            .find(|item| item.id.as_str() == id)
            .cloned()
    }

    fn provider(&self, id: &str) -> Option<Provider> {
        self.providers
            .iter()
            .find(|item| item.id.as_str() == id)
            .cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
[[provider]]
id = "anthropic"
kind = "anthropic"
endpoint = "https://api.anthropic.com"
credential_ref = "provider/anthropic"

[[model]]
id = "claude-sonnet"
provider = "anthropic"
api_name = "claude-sonnet-4-20250514"
display_name = "Claude Sonnet"
context_window = 200000
max_output_tokens = 8192
capabilities = { tool_calling = true, vision = true }
"#;

    fn parse(text: &str) -> Result<FileModelCatalog, ModelConfigError> {
        FileModelCatalog::parse(text, "test")
    }

    #[test]
    fn parses_providers_and_models() {
        let catalog = parse(SAMPLE).expect("parses");
        let model = catalog.get("claude-sonnet").expect("model");
        assert_eq!(model.provider.as_str(), "anthropic");
        assert_eq!(model.context_window, 200000);
        assert!(model.capabilities.tool_calling);
        assert!(model.capabilities.vision);
        assert!(!model.capabilities.reasoning);

        let provider = catalog.provider("anthropic").expect("provider");
        assert_eq!(provider.kind, ProviderKind::Anthropic);
        assert_eq!(provider.credential_ref, "provider/anthropic");
    }

    #[test]
    fn rejects_duplicate_and_dangling_entries() {
        let duplicate = SAMPLE.replace(
            "[[model]]",
            "[[provider]]\nid = \"anthropic\"\nkind = \"anthropic\"\nendpoint = \"x\"\ncredential_ref = \"y\"\n\n[[model]]",
        );
        assert!(matches!(
            parse(&duplicate),
            Err(ModelConfigError::DuplicateProvider(_))
        ));

        let dangling = SAMPLE.replace("provider = \"anthropic\"", "provider = \"ghost\"");
        assert!(matches!(
            parse(&dangling),
            Err(ModelConfigError::UnknownProvider { .. })
        ));
    }

    #[test]
    fn reports_unreadable_file() {
        let error = FileModelCatalog::load(Path::new("/nonexistent/models.toml")).unwrap_err();
        assert!(matches!(error, ModelConfigError::Read { .. }));
    }
}
