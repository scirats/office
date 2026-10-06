pub const MAX_SLUG_LENGTH: usize = 32;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AgentId(pub String);

impl AgentId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for AgentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InstanceId(pub String);

impl InstanceId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for InstanceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentDefinition {
    pub id: AgentId,
    pub name: String,
    pub role: String,
    pub prompt: String,
    /// Tool ids from the catalog. `agent` does not know the `tool` gear; ids
    /// cross the bridge and are resolved by the composition.
    pub tools: Vec<String>,
    pub version: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentInstance {
    pub id: InstanceId,
    pub agent: AgentId,
    pub workspace: String,
    pub project: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentInstanceRecord {
    pub id: InstanceId,
    pub agent: AgentId,
    pub workspace: String,
    pub project: String,
    pub created_at: String,
}

pub fn slugify(name: &str) -> String {
    let mut slug = String::new();
    let mut pending_dash = false;

    for character in name.trim().chars() {
        let lowered = character.to_ascii_lowercase();
        if lowered.is_ascii_alphanumeric() {
            if pending_dash && !slug.is_empty() {
                slug.push('-');
            }
            pending_dash = false;
            slug.push(lowered);
        } else {
            pending_dash = true;
        }
    }

    slug.truncate(MAX_SLUG_LENGTH);
    while slug.ends_with('-') {
        slug.pop();
    }
    slug
}

pub fn is_valid_slug(slug: &str) -> bool {
    !slug.is_empty()
        && slug.len() <= MAX_SLUG_LENGTH
        && slug.starts_with(|c: char| c.is_ascii_lowercase())
        && slug
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugifies_names() {
        assert_eq!(slugify("Backend Engineer"), "backend-engineer");
        assert_eq!(slugify("  a__b  "), "a-b");
        assert_eq!(slugify("!!!"), "");
    }

    #[test]
    fn validates_slugs() {
        assert!(is_valid_slug("backend-engineer"));
        assert!(!is_valid_slug(""));
        assert!(!is_valid_slug("Backend"));
        assert!(!is_valid_slug("backend_engineer"));
    }
}
