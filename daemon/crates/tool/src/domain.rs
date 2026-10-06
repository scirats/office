#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ToolId(pub String);

impl ToolId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ToolId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// A function the model can call, in the tool-calling sense: a name, a
/// description, and a JSON Schema for its arguments (matching the `function`
/// object of the OpenAI/DeepSeek tools API).
#[derive(Debug, Clone, PartialEq)]
pub struct Tool {
    pub id: ToolId,
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}
