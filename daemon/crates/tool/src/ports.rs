use crate::domain::Tool;

pub trait ToolCatalog: Send + Sync {
    fn list(&self) -> Vec<Tool>;
    fn get(&self, id: &str) -> Option<Tool>;
}
