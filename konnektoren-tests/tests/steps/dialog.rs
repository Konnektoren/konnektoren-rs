use crate::BddWorld;
use cucumber::{given, then, when};
use konnektoren_rs::core::challenges::Solvable;
use konnektoren_rs::prelude::*;

// ── Helpers ───────────────────────────────────────────────────────────────────

fn make_quiz_dialog() -> Dialog {
    Dialog::default()
}

fn make_observer_dialog() -> Dialog {
    let mut dialog = Dialog::default();
    for turn in &mut dialog.turns {
        turn.options = None;
        turn.correct_option = None;
    }
    dialog
}

// ── Given ─────────────────────────────────────────────────────────────────────

#[given(expr = "a Dialog challenge in Quiz mode is loaded")]
async fn a_dialog_challenge_in_quiz_mode_is_loaded(world: &mut BddWorld) {
    let dialog = make_quiz_dialog();
    let challenge_type = ChallengeType::Dialog(dialog);
    let challenge = Challenge::new(&challenge_type, &ChallengeConfig::default());
    world.challenge_type = challenge_type;
    world.challenge = Some(challenge);
}

#[given(expr = "a Dialog challenge in Observer mode is loaded")]
async fn a_dialog_challenge_in_observer_mode_is_loaded(world: &mut BddWorld) {
    let dialog = make_observer_dialog();
    let challenge_type = ChallengeType::Dialog(dialog);
    let challenge = Challenge::new(&challenge_type, &ChallengeConfig::default());
    world.challenge_type = challenge_type;
    world.challenge = Some(challenge);
}

// ── When ──────────────────────────────────────────────────────────────────────

#[when(expr = "the dialog is observed")]
async fn the_dialog_is_observed(world: &mut BddWorld) {
    // Observer mode: no interaction — the result vec stays empty, score is 100 %.
    let challenge = world.challenge.as_ref().expect("no challenge loaded");
    world.challenge_result = Some(challenge.challenge_result.clone());
}

#[when(expr = "the player answers all interactive turns correctly")]
async fn the_player_answers_all_interactive_turns_correctly(world: &mut BddWorld) {
    let challenge = world.challenge.as_mut().expect("no challenge loaded");

    // Asset dialog: turn 1 correct_option=0, turn 3 correct_option=2, turn 5 correct_option=0
    let inputs = vec![(1usize, 0usize), (3usize, 2usize), (5usize, 0usize)];

    for (turn_index, selected_option) in inputs {
        let input = ChallengeInput::Dialog(DialogAnswer {
            turn_index,
            selected_option,
        });
        challenge
            .solve(input, turn_index)
            .expect("solve should not error");
    }

    world.challenge_result = Some(challenge.challenge_result.clone());
}

#[when(expr = "the player answers the first interactive turn correctly")]
async fn the_player_answers_the_first_interactive_turn_correctly(world: &mut BddWorld) {
    let challenge = world.challenge.as_mut().expect("no challenge loaded");
    // Turn 1, correct_option = 0
    let input = ChallengeInput::Dialog(DialogAnswer {
        turn_index: 1,
        selected_option: 0,
    });
    challenge.solve(input, 1).expect("solve should not error");
    world.challenge_result = Some(challenge.challenge_result.clone());
}

#[when(expr = "the player answers the second interactive turn incorrectly")]
async fn the_player_answers_the_second_interactive_turn_incorrectly(world: &mut BddWorld) {
    let challenge = world.challenge.as_mut().expect("no challenge loaded");
    // Turn 3, correct_option = 2 — deliberately pick option 0 (wrong)
    let input = ChallengeInput::Dialog(DialogAnswer {
        turn_index: 3,
        selected_option: 0,
    });
    challenge.solve(input, 3).expect("solve should not error");
    world.challenge_result = Some(challenge.challenge_result.clone());
}

#[when(expr = "the player answers interactive turn {int} with option {int}")]
async fn the_player_answers_interactive_turn_with_option(
    world: &mut BddWorld,
    turn_index: usize,
    selected_option: usize,
) {
    let challenge = world.challenge.as_mut().expect("no challenge loaded");
    let input = ChallengeInput::Dialog(DialogAnswer {
        turn_index,
        selected_option,
    });
    let result = challenge
        .solve(input, turn_index)
        .expect("solve should not error");
    world.last_solve_correct = Some(result);
    world.challenge_result = Some(challenge.challenge_result.clone());
}

#[when(expr = "the player answers turn {int} with option {int}")]
async fn the_player_answers_turn_with_option(
    world: &mut BddWorld,
    turn_index: usize,
    selected_option: usize,
) {
    let challenge = world.challenge.as_mut().expect("no challenge loaded");
    let input = ChallengeInput::Dialog(DialogAnswer {
        turn_index,
        selected_option,
    });
    match challenge.solve(input, turn_index) {
        Ok(correct) => {
            world.last_solve_correct = Some(correct);
            world.challenge_result = Some(challenge.challenge_result.clone());
        }
        Err(_) => {
            world.last_solve_correct = None;
            world.last_command_result =
                Err(konnektoren_rs::core::error::KonnektorenError::Challenge(
                    konnektoren_rs::core::challenges::ChallengeError::InvalidInput(format!(
                        "turn index {} out of bounds",
                        turn_index
                    )),
                ));
        }
    }
}

// ── Then ──────────────────────────────────────────────────────────────────────

#[then(expr = "the dialog performance should be {int}")]
async fn the_dialog_performance_should_be(world: &mut BddWorld, expected: u32) {
    let challenge = world.challenge.as_ref().expect("no challenge loaded");
    let result = world.challenge_result.as_ref().expect("no result recorded");
    let score = challenge.performance(result);
    assert_eq!(
        score, expected,
        "expected performance {}, got {}",
        expected, score
    );
}

#[then(expr = "{int} dialog answers should be recorded")]
async fn n_dialog_answers_should_be_recorded(world: &mut BddWorld, expected: usize) {
    let result = world.challenge_result.as_ref().expect("no result recorded");
    match result {
        ChallengeResult::Dialog(answers) => {
            assert_eq!(
                answers.len(),
                expected,
                "expected {} recorded answers, got {}",
                expected,
                answers.len()
            );
        }
        other => panic!("expected Dialog result, got {:?}", other),
    }
}

#[then(expr = "the answer should be correct")]
async fn the_answer_should_be_correct(world: &mut BddWorld) {
    let correct = world
        .last_solve_correct
        .expect("no solve result recorded — did you call a 'when the player answers' step?");
    assert!(correct, "expected the answer to be correct, but it was not");
}

#[then(expr = "the answer should be incorrect")]
async fn the_answer_should_be_incorrect(world: &mut BddWorld) {
    let correct = world
        .last_solve_correct
        .expect("no solve result recorded — did you call a 'when the player answers' step?");
    assert!(
        !correct,
        "expected the answer to be incorrect, but it was marked correct"
    );
}

#[then(expr = "solving the dialog turn should return an error")]
async fn solving_the_dialog_turn_should_return_an_error(world: &mut BddWorld) {
    assert!(
        world.last_solve_correct.is_none(),
        "expected an error from solve(), but got a valid result"
    );
}
