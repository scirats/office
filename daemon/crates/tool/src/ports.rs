use crate::domain::Tool;

/// Read-only view over the tools that exist in the system. Implemented in code.
pub trait ToolCatalog: Send + Sync {
    fn list(&self) -> Vec<Tool>;
    fn get(&self, id: &str) -> Option<Tool>;
}
