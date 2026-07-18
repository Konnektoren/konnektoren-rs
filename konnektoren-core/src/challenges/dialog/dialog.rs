#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::speaker::Speaker;
use super::turn::DialogTurn;

/// A two-speaker, linear conversation challenge.
///
/// A `Dialog` is a scripted exchange between exactly two named characters.
/// It is presented to the player in one of two game modes, controlled by
/// [`ChallengeVariant`]:
///
/// - **Observer** (`DialogObserver`): the player watches the exchange unfold
///   as a chat log — no interaction required, score is always **100 %**.
/// - **Quiz** (`DialogQuiz`): turns marked with `options` + `correct_option`
///   become interactive — the player picks the right line before it is
///   revealed; score is `100 * correct / total_interactive_turns`.
///
/// One YAML file can drive both modes.  The active mode is chosen in
/// [`ChallengeConfig`], not in the data itself, so the same dialog scene can
/// appear twice on the game map: once as a cut-scene and once as a scored
/// challenge.
///
/// # Data invariants
///
/// * `speakers` always contains **exactly two** entries.
/// * Any [`DialogTurn`] that has `options` **must** also carry `correct_option`.
/// * `correct_option` must be a valid index into `options`.
/// * `turn.speaker` must match one of the two [`Speaker::id`] values.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct Dialog {
    /// Unique identifier used to reference this dialog from the game map,
    /// e.g. `"dialog_beim_baecker_a1"`.
    pub id: String,

    /// Display name shown in the game UI, e.g. on the challenge card.
    pub name: String,

    /// One-sentence description of the scene shown on the challenge card.
    pub description: String,

    /// BCP-47 language code of the scripted lines, e.g. `"de"`.
    pub lang: String,

    /// Optional flavour text that sets the scene before the first turn plays.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scenario: Option<String>,

    /// FontAwesome class (e.g. `"fa-solid fa-store"`) or image URL displayed
    /// as the scene header in the game UI.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,

    /// The two characters taking part in the exchange.  Index 0 is
    /// conventionally the one who opens the conversation, but the game engine
    /// uses the `speaker` id on each turn, not the array position.
    pub speakers: [Speaker; 2],

    /// Ordered, flat sequence of turns that make up the dialog.
    /// No branching — the story always plays out in the same order.
    pub turns: Vec<DialogTurn>,
}

impl Dialog {
    /// Returns an iterator of `(turn_index, turn)` over every turn that is
    /// interactive in Quiz mode (i.e. has both `options` and `correct_option`).
    ///
    /// The index is the position in [`Dialog::turns`] and is used by
    /// [`DialogAnswer::turn_index`](super::answer::DialogAnswer::turn_index)
    /// to record which turn the player responded to.
    pub fn quiz_turns(&self) -> impl Iterator<Item = (usize, &DialogTurn)> {
        self.turns
            .iter()
            .enumerate()
            .filter(|(_, t)| t.is_quiz_turn())
    }

    /// Total number of interactive turns — the denominator of the quiz score.
    ///
    /// Returns `0` for pure Observer dialogs, which have no interactive turns.
    pub fn quiz_turn_count(&self) -> usize {
        self.quiz_turns().count()
    }

    /// Looks up a [`Speaker`] by their `id`.
    ///
    /// Returns `None` if the id does not match either character — which
    /// signals a data integrity violation in the dialog definition.
    pub fn speaker_by_id(&self, id: &str) -> Option<&Speaker> {
        self.speakers.iter().find(|s| s.id == id)
    }
}

impl Default for Dialog {
    fn default() -> Self {
        // The asset uses the `!dialog` YAML tag for ChallengeType dispatch;
        // serde_yaml ignores unknown tags when deserializing into a plain struct.
        crate::assets::default_asset("dialog_default.yml").unwrap_or_else(|| Self {
            id: String::new(),
            name: String::new(),
            description: String::new(),
            lang: String::new(),
            scenario: None,
            image: None,
            speakers: [Speaker::default(), Speaker::default()],
            turns: Vec::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── helpers ───────────────────────────────────────────────────────────────

    fn observer_dialog() -> Dialog {
        Dialog {
            id: "obs".to_string(),
            name: "Observer".to_string(),
            description: "Pure observer dialog".to_string(),
            lang: "de".to_string(),
            scenario: None,
            image: None,
            speakers: [
                Speaker {
                    id: "a".to_string(),
                    name: "A".to_string(),
                    icon: None,
                },
                Speaker {
                    id: "b".to_string(),
                    name: "B".to_string(),
                    icon: None,
                },
            ],
            turns: vec![
                DialogTurn {
                    speaker: "a".to_string(),
                    text: "Hallo!".to_string(),
                    audio: None,
                    options: None,
                    correct_option: None,
                },
                DialogTurn {
                    speaker: "b".to_string(),
                    text: "Hallo!".to_string(),
                    audio: None,
                    options: None,
                    correct_option: None,
                },
            ],
        }
    }

    fn quiz_dialog() -> Dialog {
        Dialog {
            id: "quiz".to_string(),
            name: "Quiz".to_string(),
            description: "Dialog with interactive turns".to_string(),
            lang: "de".to_string(),
            scenario: Some("Am Markt".to_string()),
            image: Some("fa-solid fa-store".to_string()),
            speakers: [
                Speaker {
                    id: "baecker".to_string(),
                    name: "Bäcker".to_string(),
                    icon: Some("fa-solid fa-bread-slice".to_string()),
                },
                Speaker {
                    id: "kunde".to_string(),
                    name: "Kunde".to_string(),
                    icon: None,
                },
            ],
            turns: vec![
                DialogTurn {
                    speaker: "baecker".to_string(),
                    text: "Guten Morgen! Was darf es sein?".to_string(),
                    audio: None,
                    options: None,
                    correct_option: None,
                },
                DialogTurn {
                    speaker: "kunde".to_string(),
                    text: "Ich hätte gerne vier Brötchen, bitte.".to_string(),
                    audio: None,
                    options: Some(vec![
                        "Ich hätte gerne vier Brötchen, bitte.".to_string(),
                        "Guten Abend! Vier Brötchen geben.".to_string(),
                        "Hallo! Was haben Sie?".to_string(),
                    ]),
                    correct_option: Some(0),
                },
                DialogTurn {
                    speaker: "baecker".to_string(),
                    text: "Gerne! Sonst noch etwas?".to_string(),
                    audio: None,
                    options: None,
                    correct_option: None,
                },
                DialogTurn {
                    speaker: "kunde".to_string(),
                    text: "Ja, ein Vollkornbrot, bitte.".to_string(),
                    audio: None,
                    options: Some(vec![
                        "Nein, das ist alles.".to_string(),
                        "Ja, ein Vollkornbrot, bitte.".to_string(),
                        "Ich weiß nicht, was das ist.".to_string(),
                    ]),
                    correct_option: Some(1),
                },
                DialogTurn {
                    speaker: "baecker".to_string(),
                    text: "Sehr gerne. Das macht zwei Euro achtzig.".to_string(),
                    audio: None,
                    options: None,
                    correct_option: None,
                },
                DialogTurn {
                    speaker: "kunde".to_string(),
                    text: "Hier, bitte. Danke schön!".to_string(),
                    audio: None,
                    options: Some(vec![
                        "Hier, bitte. Danke schön!".to_string(),
                        "Entschuldigung. Auf Wiedersehen!".to_string(),
                        "Das ist zu teuer!".to_string(),
                    ]),
                    correct_option: Some(0),
                },
                DialogTurn {
                    speaker: "baecker".to_string(),
                    text: "Danke! Auf Wiedersehen!".to_string(),
                    audio: None,
                    options: None,
                    correct_option: None,
                },
            ],
        }
    }

    // ── Default ───────────────────────────────────────────────────────────────

    #[test]
    fn default_dialog_id_and_lang() {
        let d = Dialog::default();
        assert_eq!(d.id, "dialog_begruessung");
        assert_eq!(d.lang, "de");
    }

    #[test]
    fn default_dialog_has_two_speakers() {
        let d = Dialog::default();
        assert_eq!(d.speakers.len(), 2);
    }

    #[test]
    fn default_dialog_speakers_have_distinct_ids() {
        let d = Dialog::default();
        assert_ne!(d.speakers[0].id, d.speakers[1].id);
    }

    #[test]
    fn default_dialog_has_turns() {
        let d = Dialog::default();
        assert!(!d.turns.is_empty());
    }

    #[test]
    fn default_dialog_has_three_quiz_turns() {
        // Asset dialog_default.yml has quiz turns at indices 1, 3, 5
        assert_eq!(Dialog::default().quiz_turn_count(), 3);
    }

    // ── quiz_turns / quiz_turn_count ──────────────────────────────────────────

    #[test]
    fn observer_dialog_has_no_quiz_turns() {
        assert_eq!(observer_dialog().quiz_turn_count(), 0);
    }

    #[test]
    fn quiz_dialog_has_correct_quiz_turn_count() {
        assert_eq!(quiz_dialog().quiz_turn_count(), 3);
    }

    #[test]
    fn quiz_turns_iterator_yields_correct_indices() {
        let dialog = quiz_dialog();
        let indices: Vec<usize> = dialog.quiz_turns().map(|(i, _)| i).collect();
        assert_eq!(indices, vec![1, 3, 5]);
    }

    #[test]
    fn quiz_turns_all_have_is_quiz_turn_true() {
        for (_, turn) in quiz_dialog().quiz_turns() {
            assert!(turn.is_quiz_turn());
        }
    }

    // ── speaker_by_id ─────────────────────────────────────────────────────────

    #[test]
    fn speaker_by_id_finds_first_speaker() {
        let d = Dialog::default();
        let speaker = d.speaker_by_id("anna").expect("anna should exist");
        assert_eq!(speaker.name, "Anna");
    }

    #[test]
    fn speaker_by_id_finds_second_speaker() {
        let d = Dialog::default();
        let speaker = d.speaker_by_id("ben").expect("ben should exist");
        assert_eq!(speaker.name, "Ben");
    }

    #[test]
    fn speaker_by_id_returns_none_for_unknown_id() {
        let d = Dialog::default();
        assert!(d.speaker_by_id("unknown").is_none());
    }

    // ── optional top-level fields ─────────────────────────────────────────────

    #[test]
    fn scenario_is_some_when_set() {
        let d = quiz_dialog();
        assert_eq!(d.scenario, Some("Am Markt".to_string()));
    }

    #[test]
    fn scenario_is_none_for_observer() {
        let d = observer_dialog();
        assert!(d.scenario.is_none());
    }

    #[test]
    fn image_is_some_when_set() {
        let d = quiz_dialog();
        assert!(d.image.is_some());
    }

    #[test]
    fn image_is_none_for_observer() {
        let d = observer_dialog();
        assert!(d.image.is_none());
    }

    // ── serialisation round-trips ─────────────────────────────────────────────

    #[test]
    fn roundtrip_json_default() {
        let original = Dialog::default();
        let json = serde_json::to_string(&original).expect("serialize");
        let restored: Dialog = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(original, restored);
    }

    #[test]
    fn roundtrip_json_quiz_dialog() {
        let original = quiz_dialog();
        let json = serde_json::to_string(&original).expect("serialize");
        let restored: Dialog = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(original, restored);
    }

    #[test]
    fn roundtrip_yaml_default() {
        let original = Dialog::default();
        let yaml = serde_yaml::to_string(&original).expect("serialize");
        let restored: Dialog = serde_yaml::from_str(&yaml).expect("deserialize");
        assert_eq!(original, restored);
    }

    #[test]
    fn roundtrip_yaml_quiz_dialog() {
        let original = quiz_dialog();
        let yaml = serde_yaml::to_string(&original).expect("serialize");
        let restored: Dialog = serde_yaml::from_str(&yaml).expect("deserialize");
        assert_eq!(original, restored);
    }

    #[test]
    fn scenario_and_image_absent_from_json_when_none() {
        let d = observer_dialog();
        let json = serde_json::to_string(&d).expect("serialize");
        assert!(!json.contains("scenario"));
        assert!(!json.contains("image"));
    }

    // ── YAML parsing — Observer ───────────────────────────────────────────────

    #[test]
    fn parse_observer_yaml() {
        let yaml = r#"
id: "dialog_begruessung_a0"
name: "Begrüßung am Morgen"
description: "Watch two neighbours greet each other."
lang: "de"
image: "fa-solid fa-hand-wave"
speakers:
  - id: "anna"
    name: "Anna"
    icon: "fa-solid fa-user"
  - id: "ben"
    name: "Ben"
    icon: "fa-solid fa-user"
turns:
  - speaker: "anna"
    text: "Guten Morgen, Ben!"
  - speaker: "ben"
    text: "Guten Morgen, Anna! Wie geht's?"
  - speaker: "anna"
    text: "Gut, danke! Und dir?"
  - speaker: "ben"
    text: "Auch gut, danke. Tschüss!"
"#;
        let d: Dialog = serde_yaml::from_str(yaml).expect("parse");
        assert_eq!(d.id, "dialog_begruessung_a0");
        assert_eq!(d.lang, "de");
        assert_eq!(d.speakers[0].id, "anna");
        assert_eq!(d.speakers[1].id, "ben");
        assert_eq!(d.turns.len(), 4);
        assert_eq!(d.quiz_turn_count(), 0);
        for turn in &d.turns {
            assert!(!turn.is_quiz_turn());
        }
    }

    // ── YAML parsing — Quiz ───────────────────────────────────────────────────

    #[test]
    fn parse_quiz_yaml() {
        let yaml = r#"
id: "dialog_begruessung_quiz_a0"
name: "Begrüßung — Quiz"
description: "Pick the correct reply."
lang: "de"
speakers:
  - id: "anna"
    name: "Anna"
    icon: "fa-solid fa-user"
  - id: "ben"
    name: "Ben"
    icon: "fa-solid fa-user"
turns:
  - speaker: "anna"
    text: "Guten Morgen, Ben!"
  - speaker: "ben"
    text: "Guten Morgen, Anna! Wie geht's?"
    options:
      - "Guten Morgen, Anna! Wie geht's?"
      - "Gute Nacht, Anna! Wie geht's?"
      - "Auf Wiedersehen, Anna!"
    correct_option: 0
  - speaker: "anna"
    text: "Gut, danke! Und dir?"
  - speaker: "ben"
    text: "Auch gut, danke. Tschüss!"
    options:
      - "Schlecht, danke. Auf Wiedersehen!"
      - "Auch gut, danke. Tschüss!"
      - "Ich weiß nicht. Guten Tag!"
    correct_option: 1
"#;
        let d: Dialog = serde_yaml::from_str(yaml).expect("parse");
        assert_eq!(d.id, "dialog_begruessung_quiz_a0");
        assert_eq!(d.turns.len(), 4);
        assert_eq!(d.quiz_turn_count(), 2);

        let turn1 = &d.turns[1];
        assert!(turn1.is_quiz_turn());
        assert_eq!(turn1.correct_option, Some(0));
        assert_eq!(turn1.options.as_ref().unwrap().len(), 3);

        let turn3 = &d.turns[3];
        assert!(turn3.is_quiz_turn());
        assert_eq!(turn3.correct_option, Some(1));
    }

    // ── Clone & PartialEq ─────────────────────────────────────────────────────

    #[test]
    fn clone_produces_equal_value() {
        let original = Dialog::default();
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    #[test]
    fn different_ids_are_not_equal() {
        let mut other = Dialog::default();
        other.id = "different".to_string();
        assert_ne!(Dialog::default(), other);
    }
}
