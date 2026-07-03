use crate::BddWorld;
use cucumber::{given, then, when};
use konnektoren_core::achievements::AchievementEvaluator;
use konnektoren_core::challenges::Timed;
use konnektoren_core::challenges::{Challenge, ChallengeConfig, ChallengeHistory, ChallengeType};

#[given(expr = "a user with {int} XP")]
async fn a_user_with_xp(world: &mut BddWorld, xp: u32) {
    world.session.player_profile.xp = xp;
    world.game.xp = xp;
}

#[when(expr = "the user earns {int} more XP")]
async fn the_user_earns_more_xp(world: &mut BddWorld, additional_xp: u32) {
    world.session.player_profile.xp += additional_xp;
    world.game.xp += additional_xp;

    // Re-evaluate achievements after XP change
    let achievements_data = include_str!("../../../assets/achievements.yml");
    let evaluator = AchievementEvaluator::new(achievements_data).unwrap();
    let unlocked = evaluator.evaluate(&world.game);

    // Store the unlocked achievements in the world for later assertions (clone them)
    world.unlocked_achievements = unlocked.iter().map(|&a| a.clone()).collect();
    world.achievement_notification = None;

    // If any achievements unlocked, trigger notification for the first one
    if !world.unlocked_achievements.is_empty() {
        world.achievement_notification = Some(world.unlocked_achievements[0].clone());
    }
}

#[given(expr = "a user has completed {int} challenges")]
async fn a_user_has_completed_challenges(world: &mut BddWorld, count: usize) {
    world.game.challenge_history = ChallengeHistory {
        challenges: Vec::new(),
    };

    for i in 0..count {
        let mut challenge = Challenge::new(
            &ChallengeType::default(),
            &ChallengeConfig {
                id: format!("challenge_{}", i),
                ..Default::default()
            },
        );
        challenge.start();
        challenge.update_end_time(); // Mark as completed
        world.game.challenge_history.add_challenge(challenge);
    }
}

#[given(expr = "the user has {int} achievements")]
async fn the_user_has_achievements(world: &mut BddWorld, _count: usize) {
    // This step just confirms the initial state
    let achievements_data = include_str!("../../../assets/achievements.yml");
    let evaluator = AchievementEvaluator::new(achievements_data).unwrap();
    let unlocked = evaluator.evaluate(&world.game);
    assert_eq!(unlocked.len(), 0, "User should start with no achievements");
}

#[when(expr = "the user completes {int} more challenges")]
async fn the_user_completes_more_challenges(world: &mut BddWorld, additional_count: usize) {
    let current_count = world.game.challenge_history.len();

    for i in current_count..(current_count + additional_count) {
        let mut challenge = Challenge::new(
            &ChallengeType::default(),
            &ChallengeConfig {
                id: format!("challenge_{}", i),
                ..Default::default()
            },
        );
        challenge.start();
        challenge.update_end_time(); // Mark as completed
        world.game.challenge_history.add_challenge(challenge);
    }

    // Re-evaluate achievements after adding challenges
    let achievements_data = include_str!("../../../assets/achievements.yml");
    let evaluator = AchievementEvaluator::new(achievements_data).unwrap();
    let unlocked = evaluator.evaluate(&world.game);

    // Store the unlocked achievements in the world (clone them)
    world.unlocked_achievements = unlocked.iter().map(|&a| a.clone()).collect();
    world.achievement_notification = None;

    // If any achievements unlocked, trigger notification for the first one
    if !world.unlocked_achievements.is_empty() {
        world.achievement_notification = Some(world.unlocked_achievements[0].clone());
    }
}

#[given(expr = "a user with the {string} achievement")]
async fn a_user_with_the_achievement(world: &mut BddWorld, achievement_name: String) {
    // First, set up the conditions to unlock the achievement
    if achievement_name == "XP Master" {
        world.session.player_profile.xp = 1100; // Above 1000 to trigger XP Master
        world.game.xp = 1100;
    } else if achievement_name == "Challenge Champion" {
        a_user_has_completed_challenges(world, 50).await;
    }

    // Evaluate achievements to ensure the specified one is unlocked
    let achievements_data = include_str!("../../../assets/achievements.yml");
    let evaluator = AchievementEvaluator::new(achievements_data).unwrap();
    let unlocked = evaluator.evaluate(&world.game);

    // Make sure the achievement is unlocked
    let found = unlocked.iter().any(|a| a.name == achievement_name);
    assert!(
        found,
        "Achievement '{}' should be unlocked",
        achievement_name
    );

    // Store in world (clone them)
    world.unlocked_achievements = unlocked.iter().map(|&a| a.clone()).collect();
}

#[when(expr = "the user starts a new session")]
async fn the_user_starts_a_new_session(world: &mut BddWorld) {
    // Create a new session but keep the game state
    let player_profile = world.session.player_profile.clone();
    let game_state = world.session.game_state.clone();

    world.session = konnektoren_core::session::Session::new(player_profile.id.clone());
    world.session.player_profile = player_profile;
    world.session.game_state = game_state;

    // Re-evaluate achievements for the new session
    let achievements_data = include_str!("../../../assets/achievements.yml");
    let evaluator = AchievementEvaluator::new(achievements_data).unwrap();
    let unlocked = evaluator.evaluate(&world.game);
    world.unlocked_achievements = unlocked.iter().map(|&a| a.clone()).collect();
}

#[then(expr = "the {string} achievement should be unlocked")]
async fn the_achievement_should_be_unlocked(world: &mut BddWorld, achievement_name: String) {
    let found = world
        .unlocked_achievements
        .iter()
        .any(|a| a.name == achievement_name);
    assert!(
        found,
        "Achievement '{}' should be unlocked",
        achievement_name
    );
}

#[then(expr = "the achievement count should be {int}")]
async fn the_achievement_count_should_be(world: &mut BddWorld, count: usize) {
    assert_eq!(
        world.unlocked_achievements.len(),
        count,
        "Expected {} achievements, but got {}",
        count,
        world.unlocked_achievements.len()
    );
}

#[then(expr = "the {string} achievement should still be present")]
async fn the_achievement_should_still_be_present(world: &mut BddWorld, achievement_name: String) {
    let found = world
        .unlocked_achievements
        .iter()
        .any(|a| a.name == achievement_name);
    assert!(
        found,
        "Achievement '{}' should still be present",
        achievement_name
    );
}

#[then(expr = "an achievement notification should be triggered")]
async fn an_achievement_notification_should_be_triggered(world: &mut BddWorld) {
    assert!(
        world.achievement_notification.is_some(),
        "An achievement notification should be triggered"
    );
}

#[then(expr = "the notification should contain {string}")]
async fn the_notification_should_contain(world: &mut BddWorld, achievement_name: String) {
    let notification = world
        .achievement_notification
        .as_ref()
        .expect("No achievement notification");
    assert_eq!(
        notification.name, achievement_name,
        "Notification should be for '{}'",
        achievement_name
    );
}
