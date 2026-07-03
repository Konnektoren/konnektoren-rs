#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct Vocabulary {
    /// Unique identifier for the vocabulary set
    pub id: String,
    /// Display name of the vocabulary set
    pub name: String,
    /// Description of the vocabulary set
    pub description: String,
    /// Optional icon identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    /// Language code
    pub lang: String,
    /// List of vocabulary items
    pub items: Vec<VocabularyItem>,
}

impl Default for Vocabulary {
    fn default() -> Self {
        let data = include_str!("../../../../assets/vocabulary_default.yml");
        serde_yaml::from_str(data).unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct VocabularyItem {
    /// Item identifier
    pub id: usize,
    /// The word or phrase
    pub text: String,
    /// Translation of the word
    #[serde(skip_serializing_if = "Option::is_none")]
    pub translation: Option<String>,
    /// Optional icon identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    /// Phonetic pronunciation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phonetic: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_vocabulary() {
        let id = "vocab-test".to_string();
        let name = "Test Vocabulary".to_string();
        let description = "A test vocabulary set".to_string();
        let icon = Some("fa-solid fa-book".to_string());
        let lang = "en".to_string();
        let items = vec![
            VocabularyItem {
                id: 0,
                text: "hello".to_string(),
                translation: Some("hola".to_string()),
                icon: Some("fa-solid fa-hand-wave".to_string()),
                phonetic: Some("/həˈloʊ/".to_string()),
            },
            VocabularyItem {
                id: 1,
                text: "world".to_string(),
                translation: Some("mundo".to_string()),
                icon: Some("fa-solid fa-globe".to_string()),
                phonetic: Some("/wɜːrld/".to_string()),
            },
        ];

        let vocabulary = Vocabulary {
            id: id.clone(),
            name: name.clone(),
            description: description.clone(),
            icon: icon.clone(),
            lang: lang.clone(),
            items: items.clone(),
        };

        assert_eq!(vocabulary.id, id);
        assert_eq!(vocabulary.name, name);
        assert_eq!(vocabulary.description, description);
        assert_eq!(vocabulary.icon, icon);
        assert_eq!(vocabulary.lang, lang);
        assert_eq!(vocabulary.items.len(), 2);
        assert_eq!(vocabulary.items, items);
    }

    #[test]
    fn test_vocabulary_deserialization() {
        let yaml = r#"
        id: "basic-greetings"
        name: "Basic Greetings"
        description: "Learn common greeting words"
        icon: "fa-solid fa-hand-wave"
        lang: "de"
        items:
          - id: 0
            text: "Hallo"
            translation: "Hello"
            icon: "fa-solid fa-hand-wave"
            phonetic: "/ˈhalo/"
          - id: 1
            text: "Tschüss"
            translation: "Goodbye"
            icon: "fa-solid fa-hand-peace"
            phonetic: "/tʃyːs/"
        "#;

        let vocabulary: Vocabulary = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(vocabulary.id, "basic-greetings");
        assert_eq!(vocabulary.name, "Basic Greetings");
        assert_eq!(vocabulary.description, "Learn common greeting words");
        assert_eq!(vocabulary.icon, Some("fa-solid fa-hand-wave".to_string()));
        assert_eq!(vocabulary.lang, "de");
        assert_eq!(vocabulary.items.len(), 2);

        let first_item = &vocabulary.items[0];
        assert_eq!(first_item.id, 0);
        assert_eq!(first_item.text, "Hallo");
        assert_eq!(first_item.translation, Some("Hello".to_string()));
        assert_eq!(first_item.icon, Some("fa-solid fa-hand-wave".to_string()));
        assert_eq!(first_item.phonetic, Some("/ˈhalo/".to_string()));

        let second_item = &vocabulary.items[1];
        assert_eq!(second_item.id, 1);
        assert_eq!(second_item.text, "Tschüss");
        assert_eq!(second_item.translation, Some("Goodbye".to_string()));
        assert_eq!(second_item.icon, Some("fa-solid fa-hand-peace".to_string()));
        assert_eq!(second_item.phonetic, Some("/tʃyːs/".to_string()));
    }

    #[test]
    fn test_vocabulary_item_default() {
        let item = VocabularyItem::default();
        assert_eq!(item.id, 0);
        assert_eq!(item.text, "");
        assert_eq!(item.translation, None);
        assert_eq!(item.icon, None);
        assert_eq!(item.phonetic, None);
    }

    #[test]
    fn test_vocabulary_item_partial_data() {
        let yaml = r#"
        id: "minimal-vocab"
        name: "Minimal Vocabulary"
        description: "Vocabulary with minimal data"
        lang: "en"
        items:
          - id: 0
            text: "example"
          - id: 1
            text: "test"
            translation: "prueba"
        "#;

        let vocabulary: Vocabulary = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(vocabulary.items.len(), 2);

        let first_item = &vocabulary.items[0];
        assert_eq!(first_item.text, "example");
        assert_eq!(first_item.translation, None);
        assert_eq!(first_item.icon, None);
        assert_eq!(first_item.phonetic, None);

        let second_item = &vocabulary.items[1];
        assert_eq!(second_item.text, "test");
        assert_eq!(second_item.translation, Some("prueba".to_string()));
        assert_eq!(second_item.icon, None);
        assert_eq!(second_item.phonetic, None);
    }

    #[test]
    fn default_vocabulary() {
        let vocabulary = Vocabulary::default();
        assert_eq!(vocabulary.id, "vocabulary-example");
        assert_eq!(vocabulary.name, "Basic German Vocabulary");
        assert_eq!(
            vocabulary.description,
            "Learn essential German words with pronunciation"
        );
        assert_eq!(vocabulary.icon, Some("fa-solid fa-book-open".to_string()));
        assert_eq!(vocabulary.lang, "de");
        assert!(!vocabulary.items.is_empty());
    }

    #[test]
    fn serialize_vocabulary_omits_none_fields() {
        let vocabulary = Vocabulary {
            id: "test-vocab".to_string(),
            name: "Test".to_string(),
            description: "Test vocabulary".to_string(),
            icon: None,
            lang: "en".to_string(),
            items: vec![VocabularyItem {
                id: 0,
                text: "word".to_string(),
                translation: None,
                icon: None,
                phonetic: None,
            }],
        };

        let yaml = serde_yaml::to_string(&vocabulary).unwrap();
        assert!(!yaml.contains("null"));
        assert!(!yaml.contains("icon:"));
        assert!(!yaml.contains("translation:"));
        assert!(!yaml.contains("phonetic:"));
    }

    #[test]
    fn serialize_vocabulary() {
        let vocabulary = Vocabulary {
            id: "test-vocab".to_string(),
            name: "Test".to_string(),
            description: "Test vocabulary".to_string(),
            icon: Some("fa-solid fa-test".to_string()),
            lang: "en".to_string(),
            items: vec![VocabularyItem {
                id: 0,
                text: "word".to_string(),
                translation: Some("palabra".to_string()),
                icon: None,
                phonetic: None,
            }],
        };

        let json = serde_json::to_string(&vocabulary).unwrap();
        let deserialized: Vocabulary = serde_json::from_str(&json).unwrap();
        assert_eq!(vocabulary, deserialized);
    }
}
