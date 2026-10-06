pub const MAX_SLUG_LENGTH: usize = 32;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Workspace {
    pub name: String,
    pub slug: String,
    pub user: String,
    pub home: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceRecord {
    pub name: String,
    pub slug: String,
    pub user: String,
    pub home: String,
    pub created_at: String,
    pub status: String,
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

pub fn home_path(slug: &str) -> String {
    format!("/home/{slug}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugifies_to_lowercase_with_dashes() {
        assert_eq!(slugify("Acme Corp"), "acme-corp");
        assert_eq!(slugify("  Foo__Bar  "), "foo-bar");
        assert_eq!(slugify("a--b"), "a-b");
        assert_eq!(slugify("..."), "");
    }

    #[test]
    fn accepts_valid_slugs() {
        assert!(is_valid_slug("acme"));
        assert!(is_valid_slug("acme-corp-2"));
    }

    #[test]
    fn rejects_invalid_slugs() {
        assert!(!is_valid_slug(""));
        assert!(!is_valid_slug("2acme"));
        assert!(!is_valid_slug("Acme"));
        assert!(!is_valid_slug("acme_corp"));
        assert!(!is_valid_slug(&"a".repeat(MAX_SLUG_LENGTH + 1)));
    }

    #[test]
    fn builds_home_path() {
        assert_eq!(home_path("acme"), "/home/acme");
    }
}
