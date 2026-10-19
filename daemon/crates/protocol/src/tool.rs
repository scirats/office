use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolView {
    pub id: String,
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}
