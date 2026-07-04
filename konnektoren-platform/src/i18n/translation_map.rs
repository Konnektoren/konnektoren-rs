use super::language::Language;
use super::translation::Translation;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Text-keyed translations: original text -> language code -> translated text.
///
/// This is the same shape as the `i18n:` blocks in challenge translation YAML
/// files (see [`YamlTranslationAsset`](super::YamlTranslationAsset)):
///
/// ```yaml
/// i18n:
///   "What is Konnektoren?":
///     de: "Was ist Konnektoren?"
///     uk: "Що таке Konnektoren?"
/// ```
///
/// Content items (FAQs, inbox messages, ...) can carry their translations
/// inline in this format and merge them into an [`I18nConfig`](super::I18nConfig)
/// via [`I18nConfig::merge_translation_map`](super::I18nConfig::merge_translation_map).
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(transparent)]
pub struct TranslationMap(HashMap<String, HashMap<String, String>>);

impl TranslationMap {
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the translation of `text` in `lang`, or `None` if not present.
    pub fn get(&self, text: &str, lang: &Language) -> Option<&str> {
        self.0
            .get(text)
            .and_then(|translations| translations.get(lang.code()))
            .map(String::as_str)
    }

    /// Returns the translation of `text` in `lang`, falling back to `text` itself.
    pub fn resolve<'a>(&'a self, text: &'a str, lang: &Language) -> &'a str {
        self.get(text, lang).unwrap_or(text)
    }

    pub fn insert(
        &mut self,
        text: impl Into<String>,
        lang: &Language,
        translation: impl Into<String>,
    ) {
        self.0
            .entry(text.into())
            .or_default()
            .insert(lang.code().to_string(), translation.into());
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Iterates over `(original text, language code -> translation)` entries.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &HashMap<String, String>)> {
        self.0.iter()
    }

    /// Inverts into per-language JSON objects (`lang -> { text: translation }`),
    /// the shape [`I18nConfig::merge_translation`](super::I18nConfig::merge_translation) expects.
    pub fn to_language_maps(&self) -> HashMap<String, Value> {
        let mut maps: HashMap<String, serde_json::Map<String, Value>> = HashMap::new();
        for (text, translations) in &self.0 {
            for (lang, translation) in translations {
                maps.entry(lang.clone())
                    .or_default()
                    .insert(text.clone(), Value::String(translation.clone()));
            }
        }
        maps.into_iter()
            .map(|(lang, map)| (lang, Value::Object(map)))
            .collect()
    }
}

impl From<HashMap<String, HashMap<String, String>>> for TranslationMap {
    fn from(map: HashMap<String, HashMap<String, String>>) -> Self {
        Self(map)
    }
}

impl Translation for TranslationMap {
    fn t(&self, key: &str) -> String {
        self.resolve(key, &Language::default()).to_string()
    }

    fn t_with_lang(&self, key: &str, lang: &Language) -> String {
        self.resolve(key, lang).to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::I18nConfig;

    const YAML: &str = r#"
"What is Konnektoren?":
  de: "Was ist Konnektoren?"
  uk: "Що таке Konnektoren?"
"Help":
  de: "Hilfe"
"#;

    fn sample() -> TranslationMap {
        serde_yaml::from_str(YAML).expect("Failed to parse translation map YAML")
    }

    #[test]
    fn deserializes_text_keyed_yaml() {
        let map = sample();
        assert_eq!(map.len(), 2);
        assert_eq!(
            map.get("What is Konnektoren?", &Language::from("de")),
            Some("Was ist Konnektoren?")
        );
    }

    #[test]
    fn resolve_falls_back_to_original_text() {
        let map = sample();
        assert_eq!(map.resolve("Help", &Language::from("de")), "Hilfe");
        // Language without translation entry falls back
        assert_eq!(map.resolve("Help", &Language::from("uk")), "Help");
        // Unknown text falls back
        assert_eq!(map.resolve("Unknown", &Language::from("de")), "Unknown");
    }

    #[test]
    fn implements_translation_trait() {
        let map = sample();
        assert_eq!(map.t_with_lang("Help", &Language::from("de")), "Hilfe");
        assert_eq!(map.t_with_lang("Missing", &Language::from("de")), "Missing");
    }

    #[test]
    fn insert_builds_map() {
        let mut map = TranslationMap::new();
        assert!(map.is_empty());
        map.insert("Hello", &Language::from("de"), "Hallo");
        assert_eq!(map.get("Hello", &Language::from("de")), Some("Hallo"));
    }

    #[test]
    fn to_language_maps_inverts_to_lang_first() {
        let maps = sample().to_language_maps();
        assert_eq!(maps["de"]["What is Konnektoren?"], "Was ist Konnektoren?");
        assert_eq!(maps["de"]["Help"], "Hilfe");
        assert_eq!(maps["uk"]["What is Konnektoren?"], "Що таке Konnektoren?");
        assert!(maps["uk"].get("Help").is_none());
    }

    #[test]
    fn merges_into_i18n_config() {
        let mut config = I18nConfig::default();
        config.merge_translation_map(&sample());
        assert_eq!(
            config.get_translation("Help", Some(&Language::from("de"))),
            "Hilfe"
        );
    }

    #[test]
    fn serde_round_trip() {
        let map = sample();
        let json = serde_json::to_string(&map).unwrap();
        let back: TranslationMap = serde_json::from_str(&json).unwrap();
        assert_eq!(map, back);
    }
}
