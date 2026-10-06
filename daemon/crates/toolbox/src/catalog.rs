use serde_json::json;
use tool::{Tool, ToolCatalog, ToolId};

/// The tools that exist in the system, defined in code (not via the API).
pub struct BuiltinToolCatalog;

fn tool(id: &str, description: &str, parameters: serde_json::Value) -> Tool {
    Tool {
        id: ToolId(id.to_owned()),
        name: id.to_owned(),
        description: description.to_owned(),
        parameters,
    }
}

fn builtin() -> Vec<Tool> {
    vec![
        tool(
            "shell",
            "Run a shell command inside the workspace homedir.",
            json!({
                "type": "object",
                "properties": {
                    "command": { "type": "string", "description": "The command line to run." }
                },
                "required": ["command"],
                "additionalProperties": false
            }),
        ),
        tool(
            "editor",
            "Read or write a file inside the project.",
            json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Path relative to the project root." },
                    "content": { "type": "string", "description": "Full file content to write; omit to read." }
                },
                "required": ["path"],
                "additionalProperties": false
            }),
        ),
        tool(
            "tests",
            "Run the project test suite.",
            json!({
                "type": "object",
                "properties": {
                    "filter": { "type": "string", "description": "Optional test name filter." }
                },
                "required": [],
                "additionalProperties": false
            }),
        ),
        tool(
            "database",
            "Run a read-only query against the project database.",
            json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "SQL query to execute." }
                },
                "required": ["query"],
                "additionalProperties": false
            }),
        ),
        tool(
            "browser",
            "Fetch a web page and return its contents.",
            json!({
                "type": "object",
                "properties": {
                    "url": { "type": "string", "description": "Absolute URL to fetch." }
                },
                "required": ["url"],
                "additionalProperties": false
            }),
        ),
        tool(
            "scanner",
            "Run a security scanner against the project.",
            json!({
                "type": "object",
                "properties": {
                    "target": { "type": "string", "description": "Optional path or module to scan." }
                },
                "required": [],
                "additionalProperties": false
            }),
        ),
        tool(
            "network",
            "Make a restricted HTTP request.",
            json!({
                "type": "object",
                "properties": {
                    "method": { "type": "string", "enum": ["GET", "POST", "PUT", "DELETE"] },
                    "url": { "type": "string", "description": "Absolute URL." },
                    "body": { "type": "string", "description": "Optional request body." }
                },
                "required": ["method", "url"],
                "additionalProperties": false
            }),
        ),
        tool(
            "secrets",
            "Read a short-lived secret injected for this execution.",
            json!({
                "type": "object",
                "properties": {
                    "name": { "type": "string", "description": "Secret name." }
                },
                "required": ["name"],
                "additionalProperties": false
            }),
        ),
        tool(
            "diff",
            "Produce a diff of the current candidate changes.",
            json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Optional path to limit the diff." }
                },
                "required": [],
                "additionalProperties": false
            }),
        ),
        tool(
            "static-analysis",
            "Run static analysis over the project.",
            json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Optional path to analyze." }
                },
                "required": [],
                "additionalProperties": false
            }),
        ),
        tool(
            "adr",
            "Read or write an architecture decision record.",
            json!({
                "type": "object",
                "properties": {
                    "title": { "type": "string", "description": "Decision title." },
                    "body": { "type": "string", "description": "Markdown body; omit to read." }
                },
                "required": ["title"],
                "additionalProperties": false
            }),
        ),
    ]
}

impl ToolCatalog for BuiltinToolCatalog {
    fn list(&self) -> Vec<Tool> {
        builtin()
    }

    fn get(&self, id: &str) -> Option<Tool> {
        builtin().into_iter().find(|tool| tool.id.as_str() == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists_and_finds_builtins() {
        let tools = BuiltinToolCatalog;
        assert!(tools.list().iter().any(|tool| tool.id.as_str() == "shell"));
        assert!(tools.get("shell").is_some());
        assert!(tools.get("nope").is_none());
    }

    #[test]
    fn every_tool_declares_an_object_schema() {
        for tool in BuiltinToolCatalog.list() {
            assert_eq!(
                tool.parameters["type"], "object",
                "tool {} must declare an object schema",
                tool.id
            );
        }
    }
}
