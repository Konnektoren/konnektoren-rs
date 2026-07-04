use serde::{Deserialize, Serialize};
use std::fmt;

/// A free-form content tag (e.g. `general`, `technical`).
///
/// Serializes as a plain string, so YAML stays `tags: ["general", "about"]`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(transparent)]
pub struct Tag(String);

impl Tag {
    pub fn new(tag: impl Into<String>) -> Self {
        Self(tag.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<&str> for Tag {
    fn from(tag: &str) -> Self {
        Self(tag.to_string())
    }
}

impl From<String> for Tag {
    fn from(tag: String) -> Self {
        Self(tag)
    }
}

impl PartialEq<&str> for Tag {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_as_plain_string() {
        let tag = Tag::from("general");
        assert_eq!(serde_json::to_string(&tag).unwrap(), "\"general\"");
        let back: Tag = serde_json::from_str("\"general\"").unwrap();
        assert_eq!(back, tag);
    }

    #[test]
    fn compares_with_str() {
        assert_eq!(Tag::from("general"), "general");
        assert_eq!(Tag::from("general").to_string(), "general");
    }
}
