use crate::tag::Tag;
use chrono::{DateTime, Utc};
use konnektoren_platform::i18n::{Language, TranslationMap};
use serde::{Deserialize, Serialize};

/// A single FAQ entry.
///
/// `question` and `answer` hold the default-language (English) text. Translations
/// live in the text-keyed `i18n` block — the same format as the challenge
/// translation YAML files:
///
/// ```yaml
/// question: "What is Konnektoren?"
/// answer: "Konnektoren is a ..."
/// i18n:
///   "What is Konnektoren?":
///     de: "Was ist Konnektoren?"
///   "Konnektoren is a ...":
///     de: "Konnektoren ist eine ..."
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Faq {
    pub id: String,
    #[cfg_attr(feature = "schema", schemars(with = "String"))]
    pub timestamp: DateTime<Utc>,
    pub question: String,
    pub answer: String,
    #[serde(default)]
    pub tags: Vec<Tag>,
    #[serde(default)]
    pub i18n: TranslationMap,
}

impl Faq {
    pub fn get_localized_question(&self, lang: &Language) -> &str {
        self.i18n.resolve(&self.question, lang)
    }

    pub fn get_localized_answer(&self, lang: &Language) -> &str {
        self.i18n.resolve(&self.answer, lang)
    }
}
