use super::faq::Faq;
use serde::{Deserialize, Serialize};

/// Top-level container matching `faqs.yml`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct FaqData {
    pub faqs: Vec<Faq>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Datelike, Utc};
    use konnektoren_platform::i18n::Language;

    const FAQ_YAML: &str = include_str!("../../../assets/faqs.yml");

    fn load() -> FaqData {
        serde_yaml::from_str(FAQ_YAML).expect("Failed to parse FAQ YAML")
    }

    #[test]
    fn test_load_faq_data() {
        assert!(!load().faqs.is_empty(), "FAQs should not be empty");
    }

    #[test]
    fn test_faq_structure() {
        let faq_data = load();
        let first_faq = &faq_data.faqs[0];

        assert_eq!(first_faq.id, "general-what-is-konnektoren");
        assert!(first_faq.question.contains("What is Konnektoren?"));
        assert!(first_faq.answer.contains("web3-oriented platform"));
        assert!(first_faq.tags.contains(&"general".into()));
    }

    #[test]
    fn test_faq_translations() {
        let faq_data = load();
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
        for faq in &load().faqs {
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
        let faq_data = load();

        let all_tags: Vec<_> = faq_data.faqs.iter().flat_map(|faq| &faq.tags).collect();
        assert!(all_tags.iter().any(|tag| **tag == "general"));

        for faq in &faq_data.faqs {
            assert!(!faq.tags.is_empty(), "FAQ should have at least one tag");
        }
    }

    #[test]
    fn test_translation_completeness() {
        for faq in &load().faqs {
            for (text, translations) in faq.i18n.iter() {
                assert!(
                    text == &faq.question || text == &faq.answer,
                    "i18n key should match question or answer: {}",
                    text
                );
                for (lang, translation) in translations {
                    assert!(
                        !translation.is_empty(),
                        "Translation '{}' for '{}' should not be empty",
                        lang,
                        text
                    );
                }
            }
        }
    }

    #[test]
    fn test_faq_sorting() {
        let timestamps: Vec<DateTime<Utc>> = load().faqs.iter().map(|faq| faq.timestamp).collect();

        for i in 1..timestamps.len() {
            assert!(
                timestamps[i] >= timestamps[i - 1],
                "FAQs should be in chronological order"
            );
        }
    }

    #[test]
    fn test_unique_faq_ids() {
        let mut ids = Vec::new();
        for faq in &load().faqs {
            assert!(!ids.contains(&faq.id), "Duplicate FAQ ID found: {}", faq.id);
            ids.push(faq.id.clone());
        }
    }

    #[test]
    fn test_serde_round_trip() {
        let faq_data = load();
        let json = serde_json::to_string(&faq_data).unwrap();
        let back: FaqData = serde_json::from_str(&json).unwrap();
        assert_eq!(faq_data, back);
    }
}
