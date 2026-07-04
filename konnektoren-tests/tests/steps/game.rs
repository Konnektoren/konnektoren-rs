use crate::BddWorld;
use konnektoren_rs::core::game::{Game, GamePath};

use cucumber::{given, then};

#[given(regex = "default game path is loaded")]
async fn default_game_path_is_loaded(world: &mut BddWorld) {
    let default_game_path = GamePath::default();
    world.game_path = default_game_path;
}

#[then(expr = "game path should be named {string}")]
async fn game_path_should_be_named(world: &mut BddWorld, name: String) {
    assert_eq!(world.game_path.name, name);
}

#[then(expr = "it should have at least {int} challenges")]
async fn it_should_have_at_least_challenges(world: &mut BddWorld, challenges: usize) {
    assert!(world.game_path.challenges.len() >= challenges);
}

#[given(regex = "default game is loaded")]
async fn default_game_is_loaded(world: &mut BddWorld) {
    let default_game = Game::default();

    world.game_path = default_game.game_paths[0].clone();
    world.game = default_game;
}

#[then(expr = "the challenge history should have at least {int} entry")]
async fn the_challenge_history_should_have_at_least_entry(world: &mut BddWorld, entries: usize) {
    assert!(world.game.challenge_history.len() >= entries);
}

#[then(expr = "the game should have at least {int} challenges")]
async fn the_game_should_have_at_least_challenges(world: &mut BddWorld, challenges: usize) {
    assert!(world.game.game_paths[0].challenge_ids().len() >= challenges);
}
