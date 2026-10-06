use serde::{Deserialize, Serialize};

/// Wire view of a tool, as returned by the catalog or resolved in a definition.
/// `parameters` is the JSON Schema of the function arguments.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolView {
    pub id: String,
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}
