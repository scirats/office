pub const MAX_SLUG_LENGTH: usize = 32;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProjectId(pub String);

impl ProjectId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ProjectId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Project {
    pub id: ProjectId,
    pub workspace: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectRecord {
    pub id: ProjectId,
    pub workspace: String,
    pub name: String,
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
        assert_eq!(slugify("Ecommerce Backend"), "ecommerce-backend");
        assert_eq!(slugify("  a__b  "), "a-b");
        assert_eq!(slugify("!!!"), "");
    }

    #[test]
    fn validates_slugs() {
        assert!(is_valid_slug("ecommerce"));
        assert!(!is_valid_slug(""));
        assert!(!is_valid_slug("Ecommerce"));
        assert!(!is_valid_slug("ecommerce_backend"));
    }
}
