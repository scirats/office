#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProviderId(pub String);

impl ProviderId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ProviderId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModelId(pub String);

impl ModelId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ModelId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProviderKind {
    Anthropic,
    OpenAi,
    DeepSeek,
    AzureOpenAi,
    Local,
    Other(String),
}

impl ProviderKind {
    pub fn as_str(&self) -> &str {
        match self {
            ProviderKind::Anthropic => "anthropic",
            ProviderKind::OpenAi => "openai",
            ProviderKind::DeepSeek => "deepseek",
            ProviderKind::AzureOpenAi => "azure-openai",
            ProviderKind::Local => "local",
            ProviderKind::Other(value) => value,
        }
    }

    pub fn parse(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "anthropic" => ProviderKind::Anthropic,
            "openai" => ProviderKind::OpenAi,
            "deepseek" => ProviderKind::DeepSeek,
            "azure-openai" => ProviderKind::AzureOpenAi,
            "local" => ProviderKind::Local,
            _ => ProviderKind::Other(value.trim().to_owned()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Provider {
    pub id: ProviderId,
    pub kind: ProviderKind,
    pub endpoint: String,
    pub credential_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Capabilities {
    pub tool_calling: bool,
    pub vision: bool,
    pub reasoning: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Model {
    pub id: ModelId,
    pub provider: ProviderId,
    pub api_name: String,
    pub display_name: String,
    pub context_window: u32,
    pub max_output_tokens: u32,
    pub capabilities: Capabilities,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_known_provider_kinds() {
        assert_eq!(ProviderKind::parse("anthropic"), ProviderKind::Anthropic);
        assert_eq!(ProviderKind::parse("OpenAI"), ProviderKind::OpenAi);
        assert_eq!(ProviderKind::parse("deepseek"), ProviderKind::DeepSeek);
        assert_eq!(
            ProviderKind::parse("azure-openai"),
            ProviderKind::AzureOpenAi
        );
        assert_eq!(ProviderKind::parse("local"), ProviderKind::Local);
    }

    #[test]
    fn keeps_unknown_provider_kind_open() {
        assert_eq!(
            ProviderKind::parse("acme"),
            ProviderKind::Other("acme".to_owned())
        );
        assert_eq!(ProviderKind::Other("acme".to_owned()).as_str(), "acme");
    }

    #[test]
    fn ids_display_their_value() {
        assert_eq!(ProviderId("anthropic".to_owned()).to_string(), "anthropic");
        assert_eq!(ModelId("claude".to_owned()).to_string(), "claude");
    }
}
