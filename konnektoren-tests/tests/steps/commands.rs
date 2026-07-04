use crate::BddWorld;
use cucumber::{given, then, when};
use konnektoren_rs::core::commands::{ChallengeCommand, Command, CommandTrait, GameCommand};
use konnektoren_rs::core::error::KonnektorenError;
use konnektoren_rs::core::game::GamePath;

#[when(expr = "the next challenge is requested")]
async fn the_next_challenge_is_requested(world: &mut BddWorld) {
    let command = Command::Game(GameCommand::NextChallenge);
    let state = &mut world.session.game_state;

    // CommandError is automatically converted to KonnektorenError via From implementation
    world.last_command_result = command.execute(state).map_err(KonnektorenError::Command);
}

#[when(expr = "the previous challenge is requested")]
async fn the_previous_challenge_is_requested(world: &mut BddWorld) {
    let command = Command::Game(GameCommand::PreviousChallenge);
    let state = &mut world.session.game_state;

    // CommandError is automatically converted to KonnektorenError via From implementation
    world.last_command_result = command.execute(state).map_err(KonnektorenError::Command);

    if world.session.game_state.current_challenge_index == 0 {
        assert!(world.last_command_result.is_err());
        // Use the pattern KonnektorenError::Command instead of CommandError
        assert!(matches!(
            world.last_command_result,
            Err(KonnektorenError::Command(_))
        ));
    } else {
        assert!(world.last_command_result.is_ok());
    }
}

#[given(expr = "the current challenge is the last challenge")]
async fn the_current_challenge_is_the_last_challenge(world: &mut BddWorld) {
    world.session.game_state.current_challenge_index = world.session.game_state.game.game_paths
        [world.session.game_state.current_game_path]
        .challenges
        .len()
        - 1;
}

#[given(expr = "the current task index is the last task index")]
async fn the_current_task_index_is_the_last_task_index(world: &mut BddWorld) {
    let current_game_path: &GamePath =
        &world.session.game_state.game.game_paths[world.session.game_state.current_game_path];
    let challenge_config =
        &current_game_path.challenges[world.session.game_state.current_challenge_index];
    let max_tasks = challenge_config.tasks.len();
    world.session.game_state.current_task_index = max_tasks - 1;
}

#[given(expr = "the current challenge is the first challenge")]
async fn the_current_challenge_is_the_first_challenge(world: &mut BddWorld) {
    world.session.game_state.current_challenge_index = 0;
}

#[when(expr = "the challenge is finished")]
async fn the_challenge_is_finished(world: &mut BddWorld) {
    let command = Command::Challenge(ChallengeCommand::Finish(None));
    let state = &mut world.session.game_state;
    match command.execute(state) {
        Ok(_) => {}
        Err(e) => panic!("Failed to finish challenge: {}", e),
    }
}

#[then(expr = "an error should be raised with message {string}")]
async fn an_error_should_be_raised_with_message(world: &mut BddWorld, expected_error: String) {
    match &world.last_command_result {
        Ok(_) => panic!("Expected an error, but got Ok"),
        Err(err) => {
            let error_message = err.to_string();
            assert!(
                error_message.contains(&expected_error),
                "Expected error message to contain '{}', but got '{}'",
                expected_error,
                error_message
            );
        }
    }
}

#[when(expr = "the challenge is solved with option {int}")]
async fn the_challenge_is_solved_with_option(world: &mut BddWorld, option_index: usize) {
    let command = Command::Challenge(ChallengeCommand::SolveOption(option_index));
    let state = &mut world.session.game_state;
    match command.execute(state) {
        Ok(_) => {}
        Err(e) => panic!(
            "Failed to solve challenge with option {}: {}",
            option_index, e
        ),
    }
}

#[then(expr = "the current task index should be {int}")]
async fn the_current_task_index_should_be(world: &mut BddWorld, expected_index: usize) {
    assert_eq!(world.session.game_state.current_task_index, expected_index);
}

#[when(expr = "the next task is requested")]
async fn the_next_task_is_requested(world: &mut BddWorld) {
    let command = Command::Challenge(ChallengeCommand::NextTask);
    let state = &mut world.session.game_state;

    // CommandError is automatically converted to KonnektorenError via From implementation
    world.last_command_result = command.execute(state).map_err(KonnektorenError::Command);
}

#[when(expr = "the previous task is requested")]
async fn the_previous_task_is_requested(world: &mut BddWorld) {
    let command = Command::Challenge(ChallengeCommand::PreviousTask);
    let state = &mut world.session.game_state;

    // CommandError is automatically converted to KonnektorenError via From implementation
    world.last_command_result = command.execute(state).map_err(KonnektorenError::Command);
}

#[given(expr = "the current task index is {int}")]
async fn the_current_task_index_is(world: &mut BddWorld, index: usize) {
    world.session.game_state.current_task_index = index;
}
