use super::{ChallengeResult, ContextualChoice, Custom, Performance, Placeholder};
use crate::challenges::dialog::Dialog;
use crate::challenges::informative::Informative;
use crate::challenges::multiple_choice::MultipleChoice;
use crate::challenges::ordering::Ordering;
use crate::challenges::sort_table::SortTable;
use crate::challenges::task_pattern::TaskPattern;
use crate::challenges::{gap_fill::GapFill, vocabulary::Vocabulary};
#[cfg(feature = "schema")]
use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize};
use strum_macros::{EnumIter, IntoStaticStr};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, EnumIter, IntoStaticStr)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum ChallengeType {
    MultipleChoice(MultipleChoice),
    ContextualChoice(ContextualChoice),
    GapFill(GapFill),
    SortTable(SortTable),
    Informative(Informative),
    Ordering(Ordering),
    Custom(Custom),
    Placeholder(Placeholder),
    Vocabulary(Vocabulary),
    Dialog(Dialog),
}

impl Default for ChallengeType {
    fn default() -> Self {
        let data = include_str!("../../../assets/konnektoren.yml");
        serde_yaml::from_str(data)
            .expect("embedded konnektoren.yml is valid YAML — this is a build-time error")
    }
}

#[cfg(feature = "schema")]
impl ChallengeType {
    /// Get the JSON schema for creating challenges
    pub fn schema() -> serde_json::Value {
        let schema = schema_for!(ChallengeType);
        serde_json::to_value(schema).expect("schemars Schema is always JSON-serializable")
    }

    /// Get the JSON schema for a specific challenge variant
    pub fn schema_for_variant(&self) -> serde_json::Value {
        let schema = match self {
            ChallengeType::MultipleChoice(_) => schema_for!(MultipleChoice),
            ChallengeType::ContextualChoice(_) => schema_for!(ContextualChoice),
            ChallengeType::GapFill(_) => schema_for!(GapFill),
            ChallengeType::SortTable(_) => schema_for!(SortTable),
            ChallengeType::Informative(_) => schema_for!(Informative),
            ChallengeType::Ordering(_) => schema_for!(Ordering),
            ChallengeType::Custom(_) => schema_for!(Custom),
            ChallengeType::Placeholder(_) => schema_for!(Placeholder),
            ChallengeType::Vocabulary(_) => schema_for!(Vocabulary),
            ChallengeType::Dialog(_) => schema_for!(Dialog),
        };
        serde_json::to_value(schema).expect("schemars Schema is always JSON-serializable")
    }

    /// Get the JSON schema as a pretty-printed string
    pub fn schema_json(&self) -> String {
        serde_json::to_string_pretty(&self.schema_for_variant())
            .expect("serde_json::Value is always serializable to string")
    }
}

impl ChallengeType {
    pub fn of_tasks(&self, task_pattern: &TaskPattern) -> Self {
        match self {
            ChallengeType::MultipleChoice(dataset) => {
                let selected_questions = task_pattern.select_items(&dataset.questions);
                let mut new_dataset = dataset.clone();
                new_dataset.questions = selected_questions;
                ChallengeType::MultipleChoice(new_dataset)
            }
            ChallengeType::ContextualChoice(dataset) => {
                let selected_items = task_pattern.select_items(&dataset.items);
                let mut new_dataset = dataset.clone();
                new_dataset.items = selected_items;
                ChallengeType::ContextualChoice(new_dataset)
            }
            ChallengeType::GapFill(dataset) => {
                let selected_questions = task_pattern.select_items(&dataset.questions);
                let mut new_dataset = dataset.clone();
                new_dataset.questions = selected_questions;
                ChallengeType::GapFill(new_dataset)
            }
            ChallengeType::SortTable(dataset) => {
                let selected_rows = task_pattern.select_items(&dataset.rows);
                let mut new_dataset = dataset.clone();
                new_dataset.rows = selected_rows;
                ChallengeType::SortTable(new_dataset)
            }
            ChallengeType::Informative(dataset) => ChallengeType::Informative(dataset.clone()),
            ChallengeType::Ordering(dataset) => ChallengeType::Ordering(dataset.clone()),
            ChallengeType::Custom(dataset) => {
                let mut dataset = dataset.clone();
                let ids: Vec<_> = (0..100).collect();
                let task_ids = task_pattern.select_items(&ids);
                dataset.task_ids = Some(task_ids);
                ChallengeType::Custom(dataset)
            }
            ChallengeType::Placeholder(dataset) => ChallengeType::Placeholder(dataset.clone()),
            ChallengeType::Vocabulary(dataset) => {
                let selected_items = task_pattern.select_items(&dataset.items);
                let mut new_dataset = dataset.clone();
                new_dataset.items = selected_items;
                ChallengeType::Vocabulary(new_dataset)
            }
            ChallengeType::Dialog(dataset) => ChallengeType::Dialog(dataset.clone()),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            ChallengeType::MultipleChoice(dataset) => &dataset.name,
            ChallengeType::ContextualChoice(dataset) => &dataset.name,
            ChallengeType::SortTable(dataset) => &dataset.name,
            ChallengeType::GapFill(dataset) => &dataset.name,
            ChallengeType::Informative(dataset) => &dataset.name,
            ChallengeType::Ordering(dataset) => &dataset.name,
            ChallengeType::Custom(dataset) => &dataset.name,
            ChallengeType::Placeholder(dataset) => &dataset.name,
            ChallengeType::Vocabulary(dataset) => &dataset.name,
            ChallengeType::Dialog(dataset) => &dataset.name,
        }
    }

    pub fn id(&self) -> &str {
        match self {
            ChallengeType::MultipleChoice(dataset) => &dataset.id,
            ChallengeType::ContextualChoice(dataset) => &dataset.id,
            ChallengeType::GapFill(dataset) => &dataset.id,
            ChallengeType::SortTable(dataset) => &dataset.id,
            ChallengeType::Informative(dataset) => &dataset.id,
            ChallengeType::Ordering(dataset) => &dataset.id,
            ChallengeType::Custom(dataset) => &dataset.id,
            ChallengeType::Placeholder(dataset) => &dataset.id,
            ChallengeType::Vocabulary(dataset) => &dataset.id,
            ChallengeType::Dialog(dataset) => &dataset.id,
        }
    }
}

impl Performance for ChallengeType {
    fn performance(&self, result: &ChallengeResult) -> u32 {
        match (self, result) {
            (ChallengeType::MultipleChoice(dataset), ChallengeResult::MultipleChoice(options)) => {
                if dataset.questions.is_empty() {
                    return 0;
                }
                let mut score = 0;
                for (question, option) in dataset.questions.iter().zip(options.iter()) {
                    if question.option == option.id {
                        score += 1;
                    }
                }
                100 * score / dataset.questions.len() as u32
            }
            (
                ChallengeType::ContextualChoice(dataset),
                ChallengeResult::ContextualChoice(choices),
            ) => {
                if dataset.items.is_empty() {
                    return 0;
                }
                let mut score = 0;
                for (item, choice) in dataset.items.iter().zip(choices.iter()) {
                    if item.choices.iter().zip(&choice.ids).all(|(c, &id)| {
                        c.options
                            .get(id)
                            .is_some_and(|selected| *selected == c.correct_answer)
                    }) {
                        score += 1;
                    }
                }
                100 * score / dataset.items.len() as u32
            }
            (ChallengeType::GapFill(dataset), ChallengeResult::GapFill(answers)) => {
                if dataset.questions.is_empty() {
                    return 0;
                }
                let mut score = 0;
                for (question, answer) in dataset.questions.iter().zip(answers.iter()) {
                    if question
                        .gaps
                        .iter()
                        .zip(answer.answers.iter())
                        .all(|(gap, ans)| gap.correct == *ans)
                    {
                        score += 1;
                    }
                }
                100 * score / dataset.questions.len() as u32
            }
            (ChallengeType::SortTable(dataset), ChallengeResult::SortTable(rows)) => {
                if dataset.rows.is_empty() {
                    return 0;
                }
                let mut score = 0;
                for (row, option) in dataset.rows.iter().zip(rows.iter()) {
                    if row.values.eq(option.values.as_slice()) {
                        score += 1;
                    }
                }
                100 * score / dataset.rows.len() as u32
            }
            (ChallengeType::Informative(_), _) => 100,
            (ChallengeType::Ordering(dataset), ChallengeResult::Ordering(results)) => {
                if dataset.items.is_empty() {
                    return 0;
                }
                let mut score = 0;
                for (item, result) in dataset.items.iter().zip(results.iter()) {
                    if item.correct_order == result.order {
                        score += 1;
                    }
                }
                100 * score / dataset.items.len() as u32
            }
            (ChallengeType::Custom(_), ChallengeResult::Custom(result)) => {
                (100.0 * result.performance) as u32
            }
            (ChallengeType::Placeholder(_), _) => 0,
            // Add this line to handle Vocabulary challenges
            (ChallengeType::Vocabulary(_), ChallengeResult::Vocabulary) => 100,
            (ChallengeType::Dialog(dialog), ChallengeResult::Dialog(answers)) => {
                let quiz_turns: Vec<_> = dialog.quiz_turns().collect();
                if quiz_turns.is_empty() {
                    return 100; // Observer mode — no quiz turns defined
                }
                let correct = quiz_turns
                    .iter()
                    .filter(|(i, turn)| {
                        answers.iter().any(|a| {
                            a.turn_index == *i && turn.correct_option == Some(a.selected_option)
                        })
                    })
                    .count();
                (100 * correct / quiz_turns.len()) as u32
            }
            _ => {
                tracing::warn!(
                    "Unhandled challenge type/result combination: {:?}",
                    (self, result)
                );
                0
            }
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::challenges::{
        Challenge, ChallengeConfig, ChallengeResult, ContextItemChoiceAnswers,
        MultipleChoiceOption, Question,
    };
    use strum::IntoEnumIterator;

    use chrono::{TimeZone, Utc};

    // Base timestamp for tests
    pub const BASE_TIMESTAMP: i64 = 1700000000;

    // Helper functions for test challenges
    pub fn create_successful_challenge() -> Challenge {
        let challenge_type = ChallengeType::MultipleChoice(MultipleChoice {
            id: "test".to_string(),
            name: "Test".to_string(),
            lang: "en".to_string(),
            options: vec![
                MultipleChoiceOption {
                    id: 0,
                    name: "Correct".to_string(),
                },
                MultipleChoiceOption {
                    id: 1,
                    name: "Wrong".to_string(),
                },
            ],
            questions: vec![Question {
                question: "Test Question".to_string(),
                help: "Test Help".to_string(),
                option: 0,
                image: None,
            }],
        });

        let mut challenge = Challenge::new(&challenge_type, &ChallengeConfig::default());

        // Set fixed timestamps
        challenge.start_time = Some(Utc.timestamp_opt(BASE_TIMESTAMP, 0).unwrap());

        // Add correct answer (id: 0)
        challenge.challenge_result = ChallengeResult::MultipleChoice(vec![MultipleChoiceOption {
            id: 0,
            name: "Correct".to_string(),
        }]);

        challenge.end_time = Some(Utc.timestamp_opt(BASE_TIMESTAMP + 3600, 0).unwrap()); // 1 hour later
        challenge
    }

    pub fn create_unsuccessful_challenge() -> Challenge {
        let challenge_type = ChallengeType::MultipleChoice(MultipleChoice {
            id: "test".to_string(),
            name: "Test".to_string(),
            lang: "en".to_string(),
            options: vec![
                MultipleChoiceOption {
                    id: 0,
                    name: "Correct".to_string(),
                },
                MultipleChoiceOption {
                    id: 1,
                    name: "Wrong".to_string(),
                },
            ],
            questions: vec![Question {
                question: "Test Question".to_string(),
                help: "Test Help".to_string(),
                option: 0,
                image: None,
            }],
        });

        let mut challenge = Challenge::new(&challenge_type, &ChallengeConfig::default());

        // Set fixed timestamps
        challenge.start_time = Some(Utc.timestamp_opt(BASE_TIMESTAMP + 7200, 0).unwrap()); // 2 hours after base

        // Add wrong answer (id: 1)
        challenge.challenge_result = ChallengeResult::MultipleChoice(vec![MultipleChoiceOption {
            id: 1,
            name: "Wrong".to_string(),
        }]);

        challenge.end_time = Some(Utc.timestamp_opt(BASE_TIMESTAMP + 10800, 0).unwrap()); // 3 hours after base
        challenge
    }

    #[test]
    fn test_successful_challenge_performance() {
        let challenge = create_successful_challenge();
        assert_eq!(
            challenge.performance(&challenge.challenge_result),
            100,
            "Successful challenge should have 100% performance"
        );
    }

    #[test]
    fn test_unsuccessful_challenge_performance() {
        let challenge = create_unsuccessful_challenge();
        assert_eq!(
            challenge.performance(&challenge.challenge_result),
            0,
            "Unsuccessful challenge should have 0% performance"
        );
    }

    #[test]
    fn default_challenge() {
        let challenge = ChallengeType::default();
        match challenge {
            ChallengeType::MultipleChoice(dataset) => {
                assert_eq!(dataset.name, "Konnektoren");
                assert_eq!(dataset.options.len(), 5);
                assert!(!dataset.questions.is_empty());
            }
            _ => panic!("Invalid challenge type"),
        }
    }

    #[test]
    fn new_challenge() {
        let challenge = ChallengeType::default();
        let new_challenge = challenge.of_tasks(&TaskPattern::Exact(2));
        match new_challenge {
            ChallengeType::MultipleChoice(dataset) => {
                assert_eq!(dataset.questions.len(), 2);
            }
            _ => panic!("Invalid challenge type"),
        }
    }

    #[test]
    fn challenge_performance() {
        let challenge = ChallengeType::default();
        let result = ChallengeResult::MultipleChoice(vec![
            MultipleChoiceOption {
                id: 1,
                name: "Option 1".to_string(),
            },
            MultipleChoiceOption {
                id: 2,
                name: "Option 2".to_string(),
            },
            MultipleChoiceOption {
                id: 3,
                name: "Option 3".to_string(),
            },
            MultipleChoiceOption {
                id: 4,
                name: "Option 4".to_string(),
            },
            MultipleChoiceOption {
                id: 5,
                name: "Option 5".to_string(),
            },
        ]);
        let performance = challenge.performance(&result);
        assert_eq!(performance, 0);
    }

    #[test]
    fn challenge_name() {
        let challenge = ChallengeType::default();
        let name = challenge.name();
        assert_eq!(name, "Konnektoren");
    }

    #[test]
    fn challenge_id() {
        let challenge = ChallengeType::default();
        let id = challenge.id();
        assert_eq!(id, "konnektoren");
    }

    #[test]
    fn contextual_choice_performance() {
        let challenge: ContextualChoice =
            serde_yaml::from_str(include_str!("../../../assets/konjunktiv-2.yml")).unwrap();
        let result = ChallengeResult::ContextualChoice(vec![
            ContextItemChoiceAnswers { ids: vec![0, 0] },
            ContextItemChoiceAnswers { ids: vec![1, 0] },
        ]);

        let performance = ChallengeType::ContextualChoice(challenge).performance(&result);
        assert_eq!(performance, 100);
    }

    #[test]
    fn test_strum_into_static_str() {
        assert_eq!(
            <ChallengeType as Into<&'static str>>::into(ChallengeType::MultipleChoice(
                MultipleChoice::default()
            )),
            "multiple-choice"
        );
        assert_eq!(
            <ChallengeType as Into<&'static str>>::into(ChallengeType::ContextualChoice(
                ContextualChoice::default()
            )),
            "contextual-choice"
        );
    }

    #[test]
    fn test_variant_iteration() {
        let variants: Vec<ChallengeType> = ChallengeType::iter()
            .map(|v| match v {
                ChallengeType::MultipleChoice(_) => {
                    ChallengeType::MultipleChoice(MultipleChoice::default())
                }
                ChallengeType::ContextualChoice(_) => {
                    ChallengeType::ContextualChoice(ContextualChoice::default())
                }
                _ => v,
            })
            .collect();

        assert!(!variants.is_empty());
        assert!(
            variants
                .iter()
                .any(|v| matches!(v, ChallengeType::MultipleChoice(_)))
        );
        assert!(
            variants
                .iter()
                .any(|v| matches!(v, ChallengeType::ContextualChoice(_)))
        );
    }

    #[test]
    fn test_serde_serialization() {
        let challenge = ChallengeType::MultipleChoice(MultipleChoice::default());
        let serialized = serde_yaml::to_string(&challenge).unwrap();
        assert!(serialized.contains("multiple-choice"));
    }

    #[cfg(feature = "schema")]
    #[test]
    fn test_schema() {
        let schema = ChallengeType::schema();

        // Verify it's a valid JSON object
        assert!(schema.is_object());

        // Check that it contains expected schema properties
        let schema_obj = schema.as_object().unwrap();
        assert!(schema_obj.contains_key("$schema") || schema_obj.contains_key("type"));

        // Verify it can be serialized back to JSON string
        let schema_string = serde_json::to_string(&schema).unwrap();
        assert!(!schema_string.is_empty());

        // Verify the schema contains information about our enum variants
        let schema_str = serde_json::to_string_pretty(&schema).unwrap();
        assert!(schema_str.contains("MultipleChoice") || schema_str.contains("multiple-choice"));
    }

    #[cfg(feature = "schema")]
    #[test]
    fn test_schema_for_variant() {
        let challenge = ChallengeType::default(); // This should be MultipleChoice
        let variant_schema = challenge.schema_for_variant();

        // Verify it's a valid JSON object
        assert!(variant_schema.is_object());

        // Check basic schema structure
        let schema_obj = variant_schema.as_object().unwrap();
        assert!(schema_obj.contains_key("type") || schema_obj.contains_key("properties"));

        // Verify it can be converted to string
        let schema_json = challenge.schema_json();
        assert!(!schema_json.is_empty());
        assert!(
            schema_json.contains("MultipleChoice")
                || schema_json.contains("id")
                || schema_json.contains("name")
        );
    }

    #[test]
    fn test_json_challenge_evaluation() {
        // 1. Build a MultipleChoice challenge with known questions and correct options
        let challenge_type = ChallengeType::MultipleChoice(MultipleChoice {
            id: "test-json-eval".to_string(),
            name: "JSON Evaluation Test".to_string(),
            lang: "en".to_string(),
            options: vec![
                MultipleChoiceOption {
                    id: 0,
                    name: "Option A".to_string(),
                },
                MultipleChoiceOption {
                    id: 1,
                    name: "Option B".to_string(),
                },
                MultipleChoiceOption {
                    id: 2,
                    name: "Option C".to_string(),
                },
            ],
            questions: vec![
                Question {
                    question: "Q1".to_string(),
                    help: String::new(),
                    image: None,
                    option: 0,
                },
                Question {
                    question: "Q2".to_string(),
                    help: String::new(),
                    image: None,
                    option: 2,
                },
                Question {
                    question: "Q3".to_string(),
                    help: String::new(),
                    image: None,
                    option: 1,
                },
            ],
        });

        // 2. Serialize to JSON -- the challenge must be exportable
        let json =
            serde_json::to_string(&challenge_type).expect("ChallengeType should serialize to JSON");
        assert!(!json.is_empty(), "JSON output must not be empty");

        // 3. Deserialize back -- a third party should be able to reconstruct it
        let from_json: ChallengeType =
            serde_json::from_str(&json).expect("ChallengeType should deserialize from JSON");
        assert_eq!(
            challenge_type, from_json,
            "round-trip through JSON must produce the same value"
        );

        // 4. Evaluate with ALL correct answers -> 100 %
        let all_correct = ChallengeResult::MultipleChoice(vec![
            MultipleChoiceOption {
                id: 0,
                name: "Option A".to_string(),
            }, // Q1 correct
            MultipleChoiceOption {
                id: 2,
                name: "Option C".to_string(),
            }, // Q2 correct
            MultipleChoiceOption {
                id: 1,
                name: "Option B".to_string(),
            }, // Q3 correct
        ]);
        assert_eq!(
            from_json.performance(&all_correct),
            100,
            "all correct answers should yield 100% performance"
        );

        // 5. Evaluate with ONE correct answer (first) -> 33 %
        let one_correct = ChallengeResult::MultipleChoice(vec![
            MultipleChoiceOption {
                id: 0,
                name: "Option A".to_string(),
            }, // Q1 correct
            MultipleChoiceOption {
                id: 0,
                name: "Option A".to_string(),
            }, // Q2 wrong (expected 2)
            MultipleChoiceOption {
                id: 0,
                name: "Option A".to_string(),
            }, // Q3 wrong (expected 1)
        ]);
        assert_eq!(
            from_json.performance(&one_correct),
            33,
            "one out of three correct should yield 33% performance"
        );

        // 6. Evaluate with NO correct answers -> 0 %
        let none_correct = ChallengeResult::MultipleChoice(vec![
            MultipleChoiceOption {
                id: 1,
                name: "Option B".to_string(),
            }, // Q1 wrong (expected 0)
            MultipleChoiceOption {
                id: 0,
                name: "Option A".to_string(),
            }, // Q2 wrong (expected 2)
            MultipleChoiceOption {
                id: 2,
                name: "Option C".to_string(),
            }, // Q3 wrong (expected 1)
        ]);
        assert_eq!(
            from_json.performance(&none_correct),
            0,
            "no correct answers should yield 0% performance"
        );
    }
}
