use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SecretRef(pub String);

impl SecretRef {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SecretRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Secret(String);

impl Secret {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for Secret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Secret(***)")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secret_debug_is_redacted() {
        let secret = Secret::new("super-secret-value");
        assert_eq!(format!("{secret:?}"), "Secret(***)");
        assert!(!format!("{secret:?}").contains("super-secret-value"));
        assert_eq!(secret.expose(), "super-secret-value");
    }

    #[test]
    fn secret_ref_displays_value() {
        assert_eq!(
            SecretRef("provider/anthropic".to_owned()).to_string(),
            "provider/anthropic"
        );
    }
}
