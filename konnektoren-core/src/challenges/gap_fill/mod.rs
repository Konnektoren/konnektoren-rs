#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct GapFill {
    /// Unique identifier for the challenge
    pub id: String,
    /// Display name of the challenge
    pub name: String,
    /// Description of the challenge
    pub description: String,
    /// Language code
    pub lang: String,
    /// List of gap-fill questions
    pub questions: Vec<GapFillQuestion>,
}

impl Default for GapFill {
    fn default() -> Self {
        crate::assets::default_asset("gap_fill_default.yml").unwrap_or_else(|| Self {
            id: String::new(),
            name: String::new(),
            description: String::new(),
            lang: String::new(),
            questions: Vec::new(),
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct GapFillQuestion {
    /// The sentence with gaps
    pub sentence: String,
    /// Gaps to be filled
    pub gaps: Vec<Gap>,
    /// Helpful hints
    pub hints: Vec<String>,
    /// Translation of the sentence
    pub translation: String,
    /// Explanation of the grammar rule
    pub explanation: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct Gap {
    /// Position of the gap in the sentence
    pub position: usize,
    /// Available options for this gap
    pub options: Vec<String>,
    /// The correct answer
    pub correct: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct GapFillAnswer {
    /// Index of the question being answered
    pub question_index: usize,
    /// Answers for each gap
    pub answers: Vec<String>,
}

impl GapFill {
    pub fn check_answer(&self, answer: &GapFillAnswer) -> bool {
        if let Some(question) = self.questions.get(answer.question_index) {
            if question.gaps.len() != answer.answers.len() {
                return false;
            }

            question
                .gaps
                .iter()
                .zip(answer.answers.iter())
                .all(|(gap, ans)| gap.correct == *ans)
        } else {
            false
        }
    }

    pub fn get_feedback(&self, answer: &GapFillAnswer) -> String {
        if let Some(question) = self.questions.get(answer.question_index) {
            if self.check_answer(answer) {
                format!("Correct! {}", question.explanation)
            } else {
                let mut feedback = String::from("Incorrect. Hints:\n");
                for hint in &question.hints {
                    feedback.push_str(&format!("- {}\n", hint));
                }
                feedback
            }
        } else {
            "Invalid question index".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gap_fill_deserialization() {
        let yaml = r#"
        id: "past-tense"
        name: "Past Tense Exercise"
        description: "Fill in the correct past tense forms"
        lang: "de"
        questions:
          - sentence: "Ich __ nach Berlin __ (fahren)."
            gaps:
              - position: 0
                options: ["bin", "habe", "war"]
                correct: "bin"
              - position: 1
                options: ["gefahren", "gefahrt", "fuhr"]
                correct: "gefahren"
            hints:
              - "Movement verbs use 'sein' as auxiliary"
              - "The past participle of 'fahren' is 'gefahren'"
            translation: "I went to Berlin"
            explanation: "We use 'sein' with verbs of movement and the past participle form"
        "#;

        let gap_fill: GapFill = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(gap_fill.id, "past-tense");
        assert_eq!(gap_fill.questions.len(), 1);

        let question = &gap_fill.questions[0];
        assert_eq!(question.gaps.len(), 2);
        assert_eq!(question.gaps[0].correct, "bin");
    }

    #[test]
    fn test_check_answer() {
        let gap_fill = GapFill {
            id: "test".to_string(),
            name: "Test".to_string(),
            description: "Test".to_string(),
            lang: "de".to_string(),
            questions: vec![GapFillQuestion {
                sentence: "Ich __ nach Berlin __ (fahren).".to_string(),
                gaps: vec![
                    Gap {
                        position: 0,
                        options: vec!["bin".to_string(), "habe".to_string()],
                        correct: "bin".to_string(),
                    },
                    Gap {
                        position: 1,
                        options: vec!["gefahren".to_string(), "gefahrt".to_string()],
                        correct: "gefahren".to_string(),
                    },
                ],
                hints: vec!["Test hint".to_string()],
                translation: "Test translation".to_string(),
                explanation: "Test explanation".to_string(),
            }],
        };

        let correct_answer = GapFillAnswer {
            question_index: 0,
            answers: vec!["bin".to_string(), "gefahren".to_string()],
        };

        let wrong_answer = GapFillAnswer {
            question_index: 0,
            answers: vec!["habe".to_string(), "gefahren".to_string()],
        };

        assert!(gap_fill.check_answer(&correct_answer));
        assert!(!gap_fill.check_answer(&wrong_answer));
    }

    #[test]
    fn test_get_feedback_correct_and_incorrect() {
        let gap_fill = GapFill {
            id: "test".to_string(),
            name: "Test".to_string(),
            description: "Test".to_string(),
            lang: "de".to_string(),
            questions: vec![GapFillQuestion {
                sentence: "Ich __ nach Berlin __ (fahren).".to_string(),
                gaps: vec![
                    Gap {
                        position: 0,
                        options: vec!["bin".to_string(), "habe".to_string()],
                        correct: "bin".to_string(),
                    },
                    Gap {
                        position: 1,
                        options: vec!["gefahren".to_string(), "gefahrt".to_string()],
                        correct: "gefahren".to_string(),
                    },
                ],
                hints: vec!["Test hint".to_string()],
                translation: "Test translation".to_string(),
                explanation: "Test explanation".to_string(),
            }],
        };

        let correct_answer = GapFillAnswer {
            question_index: 0,
            answers: vec!["bin".to_string(), "gefahren".to_string()],
        };
        let wrong_answer = GapFillAnswer {
            question_index: 0,
            answers: vec!["habe".to_string(), "gefahren".to_string()],
        };
        let invalid_answer = GapFillAnswer {
            question_index: 99,
            answers: vec![],
        };

        assert!(
            gap_fill
                .get_feedback(&correct_answer)
                .starts_with("Correct!")
        );
        assert!(
            gap_fill
                .get_feedback(&wrong_answer)
                .starts_with("Incorrect.")
        );
        assert_eq!(
            gap_fill.get_feedback(&invalid_answer),
            "Invalid question index"
        );
    }
}
