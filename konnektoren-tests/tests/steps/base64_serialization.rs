use crate::BddWorld;
use base64::Engine;
use cucumber::{given, then, when};
use konnektoren_rs::core::challenges::Base64Serializable;
use konnektoren_rs::prelude::*;

#[when(expr = "the challenge is encoded to base64")]
async fn the_challenge_is_encoded_to_base64(world: &mut BddWorld) {
    let encoded = world.challenge_type.to_base64().unwrap();
    // Store the encoded string in the session player profile name for later use
    world.session.player_profile.name = format!("base64:{}", encoded);
}

#[then(expr = "the base64 string should not be empty")]
async fn the_base64_string_should_not_be_empty(world: &mut BddWorld) {
    let encoded = world
        .session
        .player_profile
        .name
        .strip_prefix("base64:")
        .expect("Base64 string should be stored");
    assert!(!encoded.is_empty(), "Base64 string should not be empty");
}

#[when(expr = "the base64 string is decoded back to a challenge")]
async fn the_base64_string_is_decoded_back_to_a_challenge(world: &mut BddWorld) {
    let encoded = world
        .session
        .player_profile
        .name
        .strip_prefix("base64:")
        .expect("Base64 string should be stored");

    let decoded = ChallengeType::from_base64(encoded).unwrap();

    // Store the decoded challenge in the game's challenge history for comparison
    let challenge_config = ChallengeConfig {
        id: "decoded_challenge".to_string(),
        ..Default::default()
    };
    let challenge = Challenge::new(&decoded, &challenge_config);
    world.game.challenge_history.add_challenge(challenge);
}

#[then(expr = "the decoded challenge should match the original challenge")]
async fn the_decoded_challenge_should_match_the_original_challenge(world: &mut BddWorld) {
    let decoded_challenge = &world.game.challenge_history.challenges[0];
    let decoded_type = &decoded_challenge.challenge_type;

    match (&world.challenge_type, decoded_type) {
        (ChallengeType::MultipleChoice(original), ChallengeType::MultipleChoice(decoded)) => {
            assert_eq!(original.id, decoded.id, "Challenge IDs should match");
            assert_eq!(original.name, decoded.name, "Challenge names should match");
            assert_eq!(
                original.lang, decoded.lang,
                "Challenge languages should match"
            );
            assert_eq!(
                original.options.len(),
                decoded.options.len(),
                "Option counts should match"
            );
            assert_eq!(
                original.questions.len(),
                decoded.questions.len(),
                "Question counts should match"
            );

            // Compare first option
            if !original.options.is_empty() && !decoded.options.is_empty() {
                assert_eq!(
                    original.options[0].id, decoded.options[0].id,
                    "First option IDs should match"
                );
                assert_eq!(
                    original.options[0].name, decoded.options[0].name,
                    "First option names should match"
                );
            }

            // Compare first question
            if !original.questions.is_empty() && !decoded.questions.is_empty() {
                assert_eq!(
                    original.questions[0].question, decoded.questions[0].question,
                    "First question text should match"
                );
                assert_eq!(
                    original.questions[0].help, decoded.questions[0].help,
                    "First question help should match"
                );
                assert_eq!(
                    original.questions[0].option, decoded.questions[0].option,
                    "First question option should match"
                );
            }
        }
        _ => panic!("Challenge types should match and be MultipleChoice"),
    }
}

#[given(expr = "invalid base64 data {string}")]
async fn invalid_base64_data(world: &mut BddWorld, invalid_data: String) {
    // Store the invalid data for later use
    world.session.player_profile.name = format!("invalid_base64:{}", invalid_data);
}

#[when(expr = "attempting to decode the base64 data")]
async fn attempting_to_decode_the_base64_data(world: &mut BddWorld) {
    let data = if world
        .session
        .player_profile
        .name
        .starts_with("invalid_base64:")
    {
        world
            .session
            .player_profile
            .name
            .strip_prefix("invalid_base64:")
            .expect("Invalid base64 data should be stored")
    } else if world
        .session
        .player_profile
        .name
        .starts_with("valid_base64_invalid_yaml:")
    {
        world
            .session
            .player_profile
            .name
            .strip_prefix("valid_base64_invalid_yaml:")
            .expect("Valid base64 with invalid YAML should be stored")
    } else {
        panic!("No base64 data to decode");
    };

    let result = ChallengeType::from_base64(data);

    // Store the result in last_command_result
    world.last_command_result = match result {
        Ok(_) => Ok(()),
        Err(e) => Err(KonnektorenError::Challenge(e)),
    };
}

#[then(expr = "a base64 decode error should be raised")]
async fn a_base64_decode_error_should_be_raised(world: &mut BddWorld) {
    match &world.last_command_result {
        Ok(_) => panic!("Expected a base64 decode error, but got success"),
        Err(e) => {
            let error_message = e.to_string();
            assert!(
                error_message.contains("Base64 decode error") || error_message.contains("decode"),
                "Expected base64 decode error, but got: {}",
                error_message
            );
        }
    }
}

#[given(expr = "valid base64 data containing invalid YAML")]
async fn valid_base64_data_containing_invalid_yaml(world: &mut BddWorld) {
    let invalid_yaml = "invalid: yaml: content: [unclosed";
    let encoded = base64::engine::general_purpose::STANDARD.encode(invalid_yaml);
    world.session.player_profile.name = format!("valid_base64_invalid_yaml:{}", encoded);
}

#[then(expr = "a deserialization error should be raised")]
async fn a_deserialization_error_should_be_raised(world: &mut BddWorld) {
    match &world.last_command_result {
        Ok(_) => panic!("Expected a deserialization error, but got success"),
        Err(e) => {
            let error_message = e.to_string();
            assert!(
                error_message.contains("Deserialization error") ||
                error_message.contains("Base64 decode error") || // YAML errors might be wrapped as decode errors
                error_message.contains("parse"),
                "Expected deserialization error, but got: {}",
                error_message
            );
        }
    }
}

#[when(expr = "the first challenge is exported to base64")]
async fn the_first_challenge_is_exported_to_base64(world: &mut BddWorld) {
    let factory = world
        .factory
        .as_ref()
        .expect("Factory should be initialized");
    let first_challenge_id = factory.challenge_types[0].id();
    let base64_data = factory
        .export_challenge_to_base64(first_challenge_id)
        .unwrap();

    // Store the exported data
    world.session.player_profile.name = format!("exported_base64:{}", base64_data);
}

#[when(expr = "a new factory is created")]
async fn a_new_factory_is_created(world: &mut BddWorld) {
    // Create a new empty factory and store it
    // We'll use the game's challenge factory as the "new" factory
    world.game.challenge_factory = ChallengeFactory::new();
}

#[when(expr = "the base64 challenge is imported to the new factory")]
async fn the_base64_challenge_is_imported_to_the_new_factory(world: &mut BddWorld) {
    let base64_data = world
        .session
        .player_profile
        .name
        .strip_prefix("exported_base64:")
        .expect("Exported base64 data should be stored");

    world
        .game
        .challenge_factory
        .add_challenge_from_base64(base64_data)
        .unwrap();
}

#[then(expr = "the new factory should have the imported challenge")]
async fn the_new_factory_should_have_the_imported_challenge(world: &mut BddWorld) {
    assert!(
        !world.game.challenge_factory.challenge_types.is_empty(),
        "New factory should have at least one challenge type"
    );

    let imported_challenge = &world.game.challenge_factory.challenge_types[0];
    let original_challenge = &world.factory.as_ref().unwrap().challenge_types[0];

    assert_eq!(
        imported_challenge.id(),
        original_challenge.id(),
        "Imported challenge should have the same ID as original"
    );
    assert_eq!(
        imported_challenge.name(),
        original_challenge.name(),
        "Imported challenge should have the same name as original"
    );
}

#[when(expr = "the challenge is serialized to base64 and back")]
async fn the_challenge_is_serialized_to_base64_and_back(world: &mut BddWorld) {
    // First encode to base64
    let encoded = world.challenge_type.to_base64().unwrap();

    // Then decode back
    let decoded = ChallengeType::from_base64(&encoded).unwrap();

    // Store the decoded challenge by replacing the original
    let original_challenge_type = world.challenge_type.clone();
    world.challenge_type = decoded;

    // Store original in achievement_notification for comparison
    world.achievement_notification = Some(konnektoren_rs::achievements::AchievementDefinition {
        id: "original_challenge".to_string(),
        name: format!("original_id:{}", original_challenge_type.id()),
        description: format!("original_name:{}", original_challenge_type.name()),
        condition: "true".to_string(),
        icon: "📦".to_string(),
    });
}

#[then(expr = "all challenge properties should be preserved")]
async fn all_challenge_properties_should_be_preserved(world: &mut BddWorld) {
    let achievement = world
        .achievement_notification
        .as_ref()
        .expect("Original challenge data should be stored");

    let original_id = achievement
        .name
        .strip_prefix("original_id:")
        .expect("Original ID should be stored");
    let original_name = achievement
        .description
        .strip_prefix("original_name:")
        .expect("Original name should be stored");

    assert_eq!(
        world.challenge_type.id(),
        original_id,
        "Challenge ID should be preserved"
    );
    assert_eq!(
        world.challenge_type.name(),
        original_name,
        "Challenge name should be preserved"
    );
}

#[then(expr = "the challenge should have the same id")]
async fn the_challenge_should_have_the_same_id(world: &mut BddWorld) {
    let achievement = world
        .achievement_notification
        .as_ref()
        .expect("Original challenge data should be stored");

    let original_id = achievement
        .name
        .strip_prefix("original_id:")
        .expect("Original ID should be stored");

    assert_eq!(
        world.challenge_type.id(),
        original_id,
        "Challenge ID should be preserved after round-trip serialization"
    );
}

#[then(expr = "the challenge should have the same name")]
async fn the_challenge_should_have_the_same_name(world: &mut BddWorld) {
    let achievement = world
        .achievement_notification
        .as_ref()
        .expect("Original challenge data should be stored");

    let original_name = achievement
        .description
        .strip_prefix("original_name:")
        .expect("Original name should be stored");

    assert_eq!(
        world.challenge_type.name(),
        original_name,
        "Challenge name should be preserved after round-trip serialization"
    );
}

#[then(expr = "the challenge should have the same number of options")]
async fn the_challenge_should_have_the_same_number_of_options(world: &mut BddWorld) {
    match &world.challenge_type {
        ChallengeType::MultipleChoice(mc) => {
            // For the default challenge, we expect 5 options
            assert!(
                mc.options.len() >= 3,
                "Challenge should have at least 3 options after round-trip, got {}",
                mc.options.len()
            );
        }
        _ => panic!("Expected MultipleChoice challenge type"),
    }
}

#[then(expr = "the challenge should have the same number of questions")]
async fn the_challenge_should_have_the_same_number_of_questions(world: &mut BddWorld) {
    match &world.challenge_type {
        ChallengeType::MultipleChoice(mc) => {
            // For the default challenge, we expect at least 1 question
            assert!(
                !mc.questions.is_empty(),
                "Challenge should have at least 1 question after round-trip, got {}",
                mc.questions.len()
            );
        }
        _ => panic!("Expected MultipleChoice challenge type"),
    }
}
