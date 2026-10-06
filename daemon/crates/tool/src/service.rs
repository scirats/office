use crate::domain::Tool;
use crate::ports::ToolCatalog;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ToolError {
    #[error("unknown tool: {0}")]
    UnknownTool(String),
}

pub struct ToolService {
    catalog: Box<dyn ToolCatalog>,
}

impl ToolService {
    pub fn new(catalog: Box<dyn ToolCatalog>) -> Self {
        Self { catalog }
    }

    pub fn list(&self) -> Vec<Tool> {
        self.catalog.list()
    }

    pub fn get(&self, id: &str) -> Option<Tool> {
        self.catalog.get(id)
    }

    pub fn resolve(&self, ids: &[String]) -> Result<Vec<Tool>, ToolError> {
        ids.iter()
            .map(|id| {
                self.catalog
                    .get(id)
                    .ok_or_else(|| ToolError::UnknownTool(id.clone()))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::ToolId;

    struct FakeCatalog;

    impl ToolCatalog for FakeCatalog {
        fn list(&self) -> Vec<Tool> {
            vec![tool("shell"), tool("editor")]
        }

        fn get(&self, id: &str) -> Option<Tool> {
            self.list().into_iter().find(|item| item.id.as_str() == id)
        }
    }

    fn tool(id: &str) -> Tool {
        Tool {
            id: ToolId(id.to_owned()),
            name: id.to_owned(),
            description: format!("{id} capability"),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": [],
                "additionalProperties": false
            }),
        }
    }

    fn service() -> ToolService {
        ToolService::new(Box::new(FakeCatalog))
    }

    #[test]
    fn lists_catalog() {
        assert_eq!(service().list().len(), 2);
    }

    #[test]
    fn resolves_known_ids_in_order() {
        let resolved = service()
            .resolve(&["editor".to_owned(), "shell".to_owned()])
            .expect("resolves");
        assert_eq!(resolved[0].id.as_str(), "editor");
        assert_eq!(resolved[1].id.as_str(), "shell");
    }

    #[test]
    fn rejects_unknown_ids() {
        assert_eq!(
            service().resolve(&["nope".to_owned()]).unwrap_err(),
            ToolError::UnknownTool("nope".to_owned())
        );
    }
}
