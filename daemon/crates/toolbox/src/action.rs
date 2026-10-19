use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Action {
    pub program: String,
    pub args: Vec<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum ActionError {
    #[error("tool is not implemented yet: {0}")]
    NotImplemented(String),
    #[error("invalid arguments for tool '{name}': {message}")]
    Arguments { name: String, message: String },
}

pub fn action(tool: &str, arguments: &str) -> Result<Action, ActionError> {
    match tool {
        "shell" => {
            let parsed: Value =
                serde_json::from_str(arguments).map_err(|error| ActionError::Arguments {
                    name: tool.to_owned(),
                    message: error.to_string(),
                })?;
            let command = parsed
                .get("command")
                .and_then(Value::as_str)
                .ok_or_else(|| ActionError::Arguments {
                    name: tool.to_owned(),
                    message: "missing 'command'".to_owned(),
                })?;
            Ok(Action {
                program: "/bin/sh".to_owned(),
                args: vec!["-c".to_owned(), command.to_owned()],
            })
        }
        other => Err(ActionError::NotImplemented(other.to_owned())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_shell_action() {
        let action = action("shell", "{\"command\":\"echo hi\"}").expect("action");
        assert_eq!(action.program, "/bin/sh");
        assert_eq!(action.args, vec!["-c", "echo hi"]);
    }

    #[test]
    fn rejects_bad_arguments() {
        assert!(matches!(
            action("shell", "{}"),
            Err(ActionError::Arguments { .. })
        ));
        assert!(matches!(
            action("shell", "not json"),
            Err(ActionError::Arguments { .. })
        ));
    }

    #[test]
    fn rejects_unimplemented_tools() {
        assert!(matches!(
            action("editor", "{}"),
            Err(ActionError::NotImplemented(tool)) if tool == "editor"
        ));
    }
}
