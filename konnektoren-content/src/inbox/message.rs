use crate::tag::Tag;
use chrono::{DateTime, Utc};
use konnektoren_platform::i18n::{Language, TranslationMap};
use serde::{Deserialize, Serialize};

/// An inbox/news message shown to the player.
///
/// `content` holds the default-language text; translations live in the
/// text-keyed `i18n` block (same format as [`Faq`](crate::faq::Faq)).
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Message {
    #[serde(default)]
    pub id: Option<String>,
    #[cfg_attr(feature = "schema", schemars(with = "String"))]
    pub timestamp: DateTime<Utc>,
    pub sender: String,
    pub content: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<Tag>,
    #[serde(default, skip_serializing_if = "TranslationMap::is_empty")]
    pub i18n: TranslationMap,
}

impl Message {
    pub fn get_localized_content(&self, lang: &Language) -> &str {
        self.i18n.resolve(&self.content, lang)
    }
}
