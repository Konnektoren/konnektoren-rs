use crate::BddWorld;
use konnektoren_rs::{core::challenges::Solvable, prelude::*};

use cucumber::{given, then, when};

#[given(expr = "default challenge is loaded")]
async fn default_challenge_is_loaded(world: &mut BddWorld) {
    let default_challenge_type = ChallengeType::default();
    world.challenge_type = default_challenge_type;
}

#[then(expr = "it should be a MultipleChoice challenge named {string}")]
async fn it_should_be_a_multiple_choice_challenge_named(world: &mut BddWorld, name: String) {
    match &world.challenge_type {
        ChallengeType::MultipleChoice(dataset) => {
            assert_eq!(dataset.name, name);
        }
        _ => panic!("Invalid challenge type"),
    }
}

#[then(expr = "it should have exactly {int} options")]
async fn it_should_have_exactly_options(world: &mut BddWorld, options: usize) {
    match &world.challenge_type {
        ChallengeType::MultipleChoice(dataset) => {
            assert_eq!(dataset.options.len(), options);
        }
        _ => panic!("Invalid challenge type"),
    }
}

#[then(expr = "it should have at least {int} questions")]
async fn it_should_have_at_least_questions(world: &mut BddWorld, questions: usize) {
    match &world.challenge_type {
        ChallengeType::MultipleChoice(dataset) => {
            assert!(dataset.questions.len() >= questions);
        }
        _ => panic!("Invalid challenge type"),
    }
}

#[given(expr = "the challenge factory is initialized")]
async fn the_challenge_factory_is_initialized(world: &mut BddWorld) {
    let factory = ChallengeFactory::new();
    world.factory = Some(factory);
}

#[given(expr = "a default challenge is loaded to the factory")]
async fn a_default_challenge_is_loaded_to_the_factory(world: &mut BddWorld) {
    let challenge = ChallengeType::default();
    let factory = world.factory.as_mut().unwrap();
    factory.challenge_types.push(challenge);
}

#[when(regex = "a challenge of (.*) is created with (\\d+) questions")]
async fn a_challenge_of_is_created_with_questions(
    world: &mut BddWorld,
    challenge: String,
    tasks: usize,
) {
    let challenge_config = ChallengeConfig {
        challenge: challenge.replace("\"", ""),
        tasks: tasks.into(),
        ..Default::default()
    };
    let factory = world.factory.as_ref().unwrap();
    let challenge = factory.create_challenge(&challenge_config);
    world.challenge_type = challenge.unwrap().challenge_type.clone();
}

#[then(expr = "the challenge should have exactly {int} questions")]
async fn the_challenge_should_have_exactly_questions(world: &mut BddWorld, questions: usize) {
    match &world.challenge_type {
        ChallengeType::MultipleChoice(dataset) => {
            assert_eq!(dataset.questions.len(), questions);
        }
        _ => panic!("Invalid challenge type"),
    }
}

#[then(expr = "the challenge be identified as {string}")]
async fn the_challenge_be_identified_as(world: &mut BddWorld, id: String) {
    match &world.challenge_type {
        ChallengeType::MultipleChoice(dataset) => {
            assert_eq!(dataset.id, id);
        }
        _ => panic!("Invalid challenge type"),
    }
}

#[given(expr = "a multiple choice challenge is set up with a question of option {int}")]
async fn a_multiple_choice_challenge_is_set_up_with_a_question_of_option(
    world: &mut BddWorld,
    option: usize,
) {
    let id = "123".to_string();
    let name = "Test".to_string();
    let lang = "en".to_string();
    let options = vec![MultipleChoiceOption {
        id: option,
        name: "Option".to_string(),
    }];
    let questions = vec![Question {
        question: "Question".to_string(),
        help: "Help 1".to_string(),
        image: None,
        option,
    }];
    let dataset = MultipleChoice {
        id,
        name,
        lang,
        options,
        questions,
    };
    world.challenge_type = ChallengeType::MultipleChoice(dataset);
    let challenge = Challenge::new(&world.challenge_type, &ChallengeConfig::default());
    world.challenge = Some(challenge);
}

#[when(expr = "the multiple choice challenge is solved with option {int}")]
async fn the_multiple_choice_challenge_is_solved_with_option(world: &mut BddWorld, option: usize) {
    let challenge = world.challenge.as_mut().expect("No challenge");

    let input = ChallengeInput::MultipleChoice(MultipleChoiceOption {
        id: option,
        name: "Option".to_string(),
    });

    // Add task_index parameter (0 for first task in this test)
    let result = challenge.solve(input, 0);
    assert!(result.is_ok());

    world.challenge_result = Some(challenge.challenge_result.clone());

    world
        .game
        .challenge_history
        .add_challenge(challenge.clone());
}

#[then(expr = "the result performance should be at least {int}")]
async fn the_result_performance_should_be_at_least(world: &mut BddWorld, performance: u32) {
    let challenge_result = world.challenge_result.as_ref().unwrap();
    let challenge = &world.challenge;
    let score = challenge.as_ref().unwrap().performance(challenge_result);
    assert!(score >= performance);
}

#[given(expr = "the current challenge is {string}")]
#[then(expr = "the current challenge is {string}")]
async fn the_current_challenge_is(world: &mut BddWorld, name: String) {
    let current_challenge = world.session.game_state.current_challenge_index;
    let challenge = world.session.game_state.game.game_paths[0].challenges[current_challenge]
        .id
        .clone();
    assert_eq!(name, challenge, "Challenge name mismatch");
}
