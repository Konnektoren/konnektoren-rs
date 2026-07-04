use crate::BddWorld;
use konnektoren_rs::prelude::*;

use cucumber::given;

#[given(expr = "A new Session with id {string}")]
async fn a_new_session_with_id(world: &mut BddWorld, id: String) {
    world.session = Session::new(id);
}

#[given(expr = "A new Session with id {string} and name {string}")]
async fn a_new_session_with_id_and_name(world: &mut BddWorld, id: String, name: String) {
    let mut player_profile = PlayerProfile::new(id);
    player_profile.name = name;
    world.session = Session::new_with_profile(player_profile);
}
