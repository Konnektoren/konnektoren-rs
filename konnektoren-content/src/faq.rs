use chrono::{DateTime, Utc};
use konnektoren_platform::i18n::Language;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct FAQ {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub question: String,
    pub answer: String,
    pub tags: Vec<String>,
    #[serde(default)]
    pub i18n: HashMap<String, FAQTranslation>,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct FAQTranslation {
    pub question: String,
    pub answer: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct FAQData {
    pub faqs: Vec<FAQ>,
}

impl FAQ {
    pub fn get_localized_question(&self, lang: &Language) -> &str {
        self.i18n
            .get(lang.code())
            .map(|t| t.question.as_str())
            .unwrap_or(&self.question)
    }

    pub fn get_localized_answer(&self, lang: &Language) -> &str {
        self.i18n
            .get(lang.code())
            .map(|t| t.answer.as_str())
            .unwrap_or(&self.answer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;

    const FAQ_YAML: &str = include_str!("../../assets/faqs.yml");

    #[test]
    fn test_load_faq_data() {
        let faq_data: FAQData = serde_yaml::from_str(FAQ_YAML).expect("Failed to parse FAQ YAML");
        assert!(!faq_data.faqs.is_empty(), "FAQs should not be empty");
    }

    #[test]
    fn test_faq_structure() {
        let faq_data: FAQData = serde_yaml::from_str(FAQ_YAML).expect("Failed to parse FAQ YAML");
        let first_faq = &faq_data.faqs[0];

        assert_eq!(first_faq.id, "general-what-is-konnektoren");
        assert!(first_faq.question.contains("What is Konnektoren?"));
        assert!(first_faq.answer.contains("web3-oriented platform"));
        assert!(first_faq.tags.contains(&"general".to_string()));
    }

    #[test]
    fn test_faq_translations() {
        let faq_data: FAQData = serde_yaml::from_str(FAQ_YAML).expect("Failed to parse FAQ YAML");
        let first_faq = &faq_data.faqs[0];

        let de = Language::from_code("de");
        assert_eq!(
            first_faq.get_localized_question(&de),
            "🤔 Was ist Konnektoren?"
        );
        assert!(
            first_faq
                .get_localized_answer(&de)
                .contains("Web3-orientierte Plattform")
        );

        // Fallback to English for a language with no translation entry.
        let fr = Language::from_code("fr");
        assert_eq!(first_faq.get_localized_question(&fr), first_faq.question);
        assert_eq!(first_faq.get_localized_answer(&fr), first_faq.answer);
    }

    #[test]
    fn test_all_faqs_have_required_fields() {
        let faq_data: FAQData = serde_yaml::from_str(FAQ_YAML).expect("Failed to parse FAQ YAML");

        for faq in &faq_data.faqs {
            assert!(!faq.id.is_empty(), "FAQ ID should not be empty");
            assert!(!faq.question.is_empty(), "Question should not be empty");
            assert!(!faq.answer.is_empty(), "Answer should not be empty");
            assert!(!faq.tags.is_empty(), "Tags should not be empty");
            assert!(
                faq.timestamp.year() >= 2024,
                "Timestamp should be 2024 or later"
            );
        }
    }

    #[test]
    fn test_faq_tags() {
        let faq_data: FAQData = serde_yaml::from_str(FAQ_YAML).expect("Failed to parse FAQ YAML");

        let all_tags: Vec<&String> = faq_data.faqs.iter().flat_map(|faq| &faq.tags).collect();

        assert!(all_tags.contains(&&"general".to_string()));
        assert!(all_tags.contains(&&"technical".to_string()));

        for faq in &faq_data.faqs {
            assert!(!faq.tags.is_empty(), "FAQ should have at least one tag");
        }
    }

    #[test]
    fn test_translation_completeness() {
        let faq_data: FAQData = serde_yaml::from_str(FAQ_YAML).expect("Failed to parse FAQ YAML");

        for faq in &faq_data.faqs {
            if let Some(de_translation) = faq.i18n.get("de") {
                assert!(
                    !de_translation.question.is_empty(),
                    "German question should not be empty"
                );
                assert!(
                    !de_translation.answer.is_empty(),
                    "German answer should not be empty"
                );
            }
        }
    }

    #[test]
    fn test_faq_sorting() {
        let faq_data: FAQData = serde_yaml::from_str(FAQ_YAML).expect("Failed to parse FAQ YAML");

        let timestamps: Vec<DateTime<Utc>> =
            faq_data.faqs.iter().map(|faq| faq.timestamp).collect();

        for i in 1..timestamps.len() {
            assert!(
                timestamps[i] >= timestamps[i - 1],
                "FAQs should be in chronological order"
            );
        }
    }

    #[test]
    fn test_unique_faq_ids() {
        let faq_data: FAQData = serde_yaml::from_str(FAQ_YAML).expect("Failed to parse FAQ YAML");

        let mut ids = Vec::new();
        for faq in &faq_data.faqs {
            assert!(!ids.contains(&faq.id), "Duplicate FAQ ID found: {}", faq.id);
            ids.push(faq.id.clone());
        }
    }
}
