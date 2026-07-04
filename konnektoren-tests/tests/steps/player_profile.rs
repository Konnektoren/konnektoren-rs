use crate::BddWorld;
use konnektoren_rs::prelude::*;

use cucumber::{given, then, when};

#[then(expr = "player profile name is {string}")]
async fn player_profile_name_is(world: &mut BddWorld, name: String) {
    assert_eq!(world.session.player_profile.name, name);
}

#[then(expr = "player profile id is {string}")]
async fn player_profile_id_is(world: &mut BddWorld, id: String) {
    assert_eq!(world.session.player_profile.id, id);
}

#[then(expr = "player profile xp is {int}")]
async fn player_profile_xp_is(world: &mut BddWorld, xp: Xp) {
    assert_eq!(world.session.player_profile.xp, xp);
}

#[when(expr = "the player profile name is updated to {string}")]
async fn the_player_profile_name_is_updated_to(world: &mut BddWorld, new_name: String) {
    world.session.player_profile.name = new_name;
}

#[given(expr = "the player profile xp is {int}")]
async fn the_player_profile_xp_is(world: &mut BddWorld, xp: Xp) {
    world.session.player_profile.xp = xp;
}

#[when(expr = "the player profile xp is increased by {int}")]
async fn the_player_profile_xp_is_increased_by(world: &mut BddWorld, increase: Xp) {
    world.session.player_profile.xp += increase;
}

#[then(expr = "the player profile id should be {string}")]
async fn the_player_profile_id_should_be(world: &mut BddWorld, expected_id: String) {
    assert_eq!(world.session.player_profile.id, expected_id);
}

#[then(expr = "the player profile name should be {string}")]
async fn the_player_profile_name_should_be(world: &mut BddWorld, expected_name: String) {
    assert_eq!(world.session.player_profile.name, expected_name);
}

#[then(expr = "the player profile xp should be {int}")]
async fn the_player_profile_xp_should_be(world: &mut BddWorld, expected_xp: Xp) {
    assert_eq!(world.session.player_profile.xp, expected_xp);
}
