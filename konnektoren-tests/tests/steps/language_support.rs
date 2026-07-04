use crate::BddWorld;
use cucumber::{given, then, when};
use konnektoren_rs::i18n::{I18nConfig, JsonTranslationAsset, Language};
use konnektoren_rs::prelude::*;
use rust_embed::RustEmbed;
use serde_json::{Value, json};
use std::collections::HashMap;

// Define our test assets
#[derive(RustEmbed)]
#[folder = "$CARGO_MANIFEST_DIR/assets/i18n/"]
pub struct TestI18nAssets;

#[given(expr = "a challenge with language set to {string}")]
async fn a_challenge_with_language_set_to(world: &mut BddWorld, language_code: String) {
    // Use the platform's language support
    let language = Language::from_code(&language_code);

    // Load translations from the platform assets
    let mut i18n_config =
        I18nConfig::with_assets(JsonTranslationAsset::<konnektoren_rs::i18n::I18nAssets>::new());

    // Add test-specific translations
    if language_code == "de" {
        i18n_config.merge_translation(
            &language,
            json!({
                "Option": "Option",
                "Question": "Frage",
                "Help text": "Hilfetext auf Deutsch",
                "Language Test": "Sprachtest",
                "This is a test challenge": "Dies ist ein Testlauf"
            }),
        );
    } else {
        i18n_config.merge_translation(
            &language,
            json!({
                "Option": "Option",
                "Question": "Question",
                "Help text": "Help text in English",
                "Language Test": "Language Test",
                "This is a test challenge": "This is a test challenge"
            }),
        );
    }

    // Create language-specific content for the challenge
    let options = vec![
        MultipleChoiceOption {
            id: 0,
            name: format!("{} {}", i18n_config.t_with_lang("Option", &language), "1"),
        },
        MultipleChoiceOption {
            id: 1,
            name: format!("{} {}", i18n_config.t_with_lang("Option", &language), "2"),
        },
        MultipleChoiceOption {
            id: 2,
            name: format!("{} {}", i18n_config.t_with_lang("Option", &language), "3"),
        },
    ];

    let questions = vec![
        Question {
            question: format!("{} 1", i18n_config.t_with_lang("Question", &language)),
            help: i18n_config.t_with_lang("Help text", &language),
            image: None,
            option: 0,
        },
        Question {
            question: format!("{} 2", i18n_config.t_with_lang("Question", &language)),
            help: i18n_config.t_with_lang("Help text", &language),
            image: None,
            option: 1,
        },
    ];

    let mc_dataset = MultipleChoice {
        id: "language_test".to_string(),
        name: i18n_config.t_with_lang("Language Test", &language),
        lang: language_code.clone(),
        options,
        questions,
    };

    world.challenge_type = ChallengeType::MultipleChoice(mc_dataset);

    let challenge_config = ChallengeConfig {
        id: "language_test".to_string(),
        name: format!(
            "{} in {}",
            i18n_config.t_with_lang("Language Test", &language),
            language.native_name()
        ),
        description: format!(
            "{} in {}",
            i18n_config.t_with_lang("This is a test challenge", &language),
            language.native_name()
        ),
        challenge: "language_test".to_string(),
        tasks: 2.into(),
        unlock_points: 0,
        variant: None,
        position: None,
        icon: None,
    };

    let challenge = Challenge::new(&world.challenge_type, &challenge_config);

    // Store current language and i18n config directly in BddWorld
    // We'll store these as string key-value pairs
    if world.session.player_profile.name.is_empty() {
        // Use the name field to store the current language
        world.session.player_profile.name = format!("current_language:{}", language_code.clone());
    }

    // Serialize the i18n config to a JSON string and store it
    let i18n_serialized = serde_json::to_string(&i18n_config.translations).unwrap();

    // Remember the i18n data directly in the BddWorld struct
    world.achievement_notification = Some(konnektoren_rs::achievements::AchievementDefinition {
        id: "i18n_data".to_string(),
        name: i18n_serialized,
        description: language_code,
        condition: "total_xp > 1".to_string(),
        icon: "🌐".to_string(),
    });

    world.challenge = Some(challenge);
}

#[given(expr = "a challenge in German")]
async fn a_challenge_in_german(world: &mut BddWorld) {
    a_challenge_with_language_set_to(world, "de".to_string()).await;
}

#[when(expr = "the challenge is loaded")]
async fn the_challenge_is_loaded(world: &mut BddWorld) {
    // This is basically a no-op because we already created the challenge
    // in the "given" step, but we're making sure it exists
    assert!(world.challenge.is_some(), "Challenge should be loaded");
}

// Helper function to extract i18n config from BddWorld
fn get_i18n_data(world: &BddWorld) -> (I18nConfig, String) {
    // Get the achievement which stores our i18n data
    let achievement = world
        .achievement_notification
        .as_ref()
        .expect("i18n data should be stored in achievement_notification");

    let i18n_serialized = &achievement.name;
    let current_language = &achievement.description;

    let translations: HashMap<String, Value> =
        serde_json::from_str(i18n_serialized).expect("Failed to parse i18n serialized data");

    let mut i18n_config = I18nConfig::default();
    for (lang, trans) in translations {
        i18n_config.merge_translation(&Language::from_code(&lang), trans);
    }

    (i18n_config, current_language.clone())
}

#[when(expr = "the language is changed to English")]
async fn the_language_is_changed_to_english(world: &mut BddWorld) {
    // Get the current challenge
    let challenge = world.challenge.as_ref().expect("Challenge should exist");

    // Get i18n data
    let (i18n_config, _) = get_i18n_data(world);

    // Get the English language
    let english = Language::from_code("en");

    // Create a new challenge with English language
    match &world.challenge_type {
        ChallengeType::MultipleChoice(mc_dataset) => {
            // Create new language-specific content
            let options = vec![
                MultipleChoiceOption {
                    id: 0,
                    name: format!("{} {}", i18n_config.t_with_lang("Option", &english), "1"),
                },
                MultipleChoiceOption {
                    id: 1,
                    name: format!("{} {}", i18n_config.t_with_lang("Option", &english), "2"),
                },
                MultipleChoiceOption {
                    id: 2,
                    name: format!("{} {}", i18n_config.t_with_lang("Option", &english), "3"),
                },
            ];

            let questions = vec![
                Question {
                    question: format!("{} 1", i18n_config.t_with_lang("Question", &english)),
                    help: i18n_config.t_with_lang("Help text", &english),
                    image: None,
                    option: 0,
                },
                Question {
                    question: format!("{} 2", i18n_config.t_with_lang("Question", &english)),
                    help: i18n_config.t_with_lang("Help text", &english),
                    image: None,
                    option: 1,
                },
            ];

            let new_dataset = MultipleChoice {
                id: mc_dataset.id.clone(),
                name: i18n_config.t_with_lang("Language Test", &english),
                lang: "en".to_string(),
                options,
                questions,
            };

            world.challenge_type = ChallengeType::MultipleChoice(new_dataset);

            let challenge_config = ChallengeConfig {
                id: challenge.challenge_config.id.clone(),
                name: format!(
                    "{} in {}",
                    i18n_config.t_with_lang("Language Test", &english),
                    english.native_name()
                ),
                description: format!(
                    "{} in {}",
                    i18n_config.t_with_lang("This is a test challenge", &english),
                    english.native_name()
                ),
                challenge: challenge.challenge_config.challenge.clone(),
                tasks: challenge.challenge_config.tasks.clone(),
                unlock_points: challenge.challenge_config.unlock_points,
                variant: challenge.challenge_config.variant.clone(),
                position: challenge.challenge_config.position,
                icon: challenge.challenge_config.icon.clone(),
            };

            let new_challenge = Challenge::new(&world.challenge_type, &challenge_config);

            // Update the language to English
            if let Some(achievement) = &mut world.achievement_notification {
                achievement.description = "en".to_string();
            }

            // Update player profile name if it contains language info
            if world
                .session
                .player_profile
                .name
                .starts_with("current_language:")
            {
                world.session.player_profile.name = "current_language:en".to_string();
            }

            world.challenge = Some(new_challenge);
        }
        _ => panic!("Expected MultipleChoice challenge type"),
    }
}

#[then(expr = "all text elements should be in German")]
async fn all_text_elements_should_be_in_german(world: &mut BddWorld) {
    let challenge = world.challenge.as_ref().expect("Challenge should exist");

    // Get i18n data
    let (i18n_config, current_language) = get_i18n_data(world);

    assert_eq!(current_language, "de", "Current language should be German");

    let german = Language::from_code("de");

    match &world.challenge_type {
        ChallengeType::MultipleChoice(mc_dataset) => {
            // Verify language is set to German
            assert_eq!(mc_dataset.lang, "de", "Challenge language should be German");

            // Check question text is in German
            for question in &mc_dataset.questions {
                let question_text = i18n_config.t_with_lang("Question", &german);
                assert!(
                    question.question.contains(&question_text),
                    "Question text should be in German: {}, expected to contain: {}",
                    question.question,
                    question_text
                );

                let help_text = i18n_config.t_with_lang("Help text", &german);
                assert_eq!(
                    question.help, help_text,
                    "Question help should be in German: {}",
                    question.help
                );
            }

            // Check options are in German
            for option in &mc_dataset.options {
                let option_text = i18n_config.t_with_lang("Option", &german);
                assert!(
                    option.name.contains(&option_text),
                    "Option text should be in German: {}, expected to contain: {}",
                    option.name,
                    option_text
                );
            }

            // Verify challenge config text is in German
            let language_test = i18n_config.t_with_lang("Language Test", &german);
            assert!(
                challenge.challenge_config.name.contains(&language_test),
                "Challenge name should contain '{}': {}",
                language_test,
                challenge.challenge_config.name
            );

            let test_challenge = i18n_config.t_with_lang("This is a test challenge", &german);
            assert!(
                challenge
                    .challenge_config
                    .description
                    .contains(&test_challenge),
                "Challenge description should contain '{}': {}",
                test_challenge,
                challenge.challenge_config.description
            );
        }
        _ => panic!("Expected MultipleChoice challenge type"),
    }
}

#[then(expr = "all text elements should be in English")]
async fn all_text_elements_should_be_in_english(world: &mut BddWorld) {
    let challenge = world.challenge.as_ref().expect("Challenge should exist");

    // Get i18n data
    let (i18n_config, current_language) = get_i18n_data(world);

    assert_eq!(current_language, "en", "Current language should be English");

    let english = Language::from_code("en");

    match &world.challenge_type {
        ChallengeType::MultipleChoice(mc_dataset) => {
            // Verify language is set to English
            assert_eq!(
                mc_dataset.lang, "en",
                "Challenge language should be English"
            );

            // Check question text is in English
            for question in &mc_dataset.questions {
                let question_text = i18n_config.t_with_lang("Question", &english);
                assert!(
                    question.question.contains(&question_text),
                    "Question text should be in English: {}, expected to contain: {}",
                    question.question,
                    question_text
                );

                let help_text = i18n_config.t_with_lang("Help text", &english);
                assert_eq!(
                    question.help, help_text,
                    "Question help should be in English: {}",
                    question.help
                );
            }

            // Check options are in English
            for option in &mc_dataset.options {
                let option_text = i18n_config.t_with_lang("Option", &english);
                assert!(
                    option.name.contains(&option_text),
                    "Option text should be in English: {}, expected to contain: {}",
                    option.name,
                    option_text
                );
            }

            // Verify challenge config text is in English
            let language_test = i18n_config.t_with_lang("Language Test", &english);
            assert!(
                challenge.challenge_config.name.contains(&language_test),
                "Challenge name should contain '{}': {}",
                language_test,
                challenge.challenge_config.name
            );

            let test_challenge = i18n_config.t_with_lang("This is a test challenge", &english);
            assert!(
                challenge
                    .challenge_config
                    .description
                    .contains(&test_challenge),
                "Challenge description should contain '{}': {}",
                test_challenge,
                challenge.challenge_config.description
            );
        }
        _ => panic!("Expected MultipleChoice challenge type"),
    }
}

#[then(expr = "question help text should be in German")]
async fn question_help_text_should_be_in_german(world: &mut BddWorld) {
    let _challenge = world.challenge.as_ref().expect("Challenge should exist");

    // Get i18n data
    let (i18n_config, current_language) = get_i18n_data(world);

    assert_eq!(current_language, "de", "Current language should be German");

    let german = Language::from_code("de");

    match &world.challenge_type {
        ChallengeType::MultipleChoice(mc_dataset) => {
            let help_text = i18n_config.t_with_lang("Help text", &german);

            for question in &mc_dataset.questions {
                assert_eq!(
                    question.help, help_text,
                    "Question help should be in German: {}, expected: {}",
                    question.help, help_text
                );
            }
        }
        _ => panic!("Expected MultipleChoice challenge type"),
    }
}

#[then(expr = "question help text should be in English")]
async fn question_help_text_should_be_in_english(world: &mut BddWorld) {
    let _challenge = world.challenge.as_ref().expect("Challenge should exist");

    // Get i18n data
    let (i18n_config, current_language) = get_i18n_data(world);

    assert_eq!(current_language, "en", "Current language should be English");

    let english = Language::from_code("en");

    match &world.challenge_type {
        ChallengeType::MultipleChoice(mc_dataset) => {
            let help_text = i18n_config.t_with_lang("Help text", &english);

            for question in &mc_dataset.questions {
                assert_eq!(
                    question.help, help_text,
                    "Question help should be in English: {}, expected: {}",
                    question.help, help_text
                );
            }
        }
        _ => panic!("Expected MultipleChoice challenge type"),
    }
}

#[then(expr = "the challenge should be displayed in English")]
async fn the_challenge_should_be_displayed_in_english(world: &mut BddWorld) {
    all_text_elements_should_be_in_english(world).await;
}

#[then(expr = "the meaning should be preserved")]
async fn the_meaning_should_be_preserved(world: &mut BddWorld) {
    let challenge = world.challenge.as_ref().expect("Challenge should exist");

    match &world.challenge_type {
        ChallengeType::MultipleChoice(mc_dataset) => {
            // Check that we have the same number of questions and options
            assert_eq!(mc_dataset.questions.len(), 2, "Should have 2 questions");
            assert_eq!(mc_dataset.options.len(), 3, "Should have 3 options");

            // Check that the correct answer indices are preserved
            assert_eq!(
                mc_dataset.questions[0].option, 0,
                "First question should have option 0 as correct"
            );
            assert_eq!(
                mc_dataset.questions[1].option, 1,
                "Second question should have option 1 as correct"
            );

            // Check that the question numbers are preserved
            assert!(
                mc_dataset.questions[0].question.contains("1"),
                "First question should still be question 1"
            );
            assert!(
                mc_dataset.questions[1].question.contains("2"),
                "Second question should still be question 2"
            );

            // Check that the option numbers are preserved
            assert!(
                mc_dataset.options[0].name.contains("1"),
                "First option should still be option 1"
            );
            assert!(
                mc_dataset.options[1].name.contains("2"),
                "Second option should still be option 2"
            );
            assert!(
                mc_dataset.options[2].name.contains("3"),
                "Third option should still be option 3"
            );

            // Check that challenge config maintains the same structure
            assert!(
                challenge.challenge_config.id == "language_test",
                "Challenge ID should be preserved"
            );
            assert!(
                challenge.challenge_config.tasks.len() == 2,
                "Challenge should have 2 tasks"
            );
        }
        _ => panic!("Expected MultipleChoice challenge type"),
    }
}
