use crate::BddWorld;
use cucumber::{given, then, when};
use konnektoren_rs::core::challenges::{
    Challenge, ChallengeConfig, ChallengeResult, ChallengeType, Choice, ContextItem,
    ContextItemChoiceAnswers, ContextualChoice, Informative, Ordering, OrderingItem,
    OrderingResult, Performance, PerformanceRecord, Solvable, SortTable, SortTableColumn,
    SortTableRow, Timed, Vocabulary, VocabularyItem,
};
use konnektoren_rs::prelude::*;

// ============================================================================
// Background Steps
// ============================================================================

#[given(expr = "the system tracks best performance for each challenge")]
async fn system_tracks_best_performance(_world: &mut BddWorld) {
    // Documentation step - this is how the system works
}

// ============================================================================
// Vocabulary Challenge Steps
// ============================================================================

#[given(expr = "a user starts a vocabulary challenge")]
async fn user_starts_vocabulary_challenge(world: &mut BddWorld) {
    let vocabulary = Vocabulary {
        id: "test-vocab".to_string(),
        name: "Test Vocabulary".to_string(),
        description: "Test".to_string(),
        icon: None,
        lang: "en".to_string(),
        items: vec![VocabularyItem {
            id: 0,
            text: "test".to_string(),
            translation: Some("test".to_string()),
            icon: None,
            phonetic: None,
        }],
    };

    world.challenge_type = ChallengeType::Vocabulary(vocabulary);
    let challenge = Challenge::new(&world.challenge_type, &ChallengeConfig::default());
    world.challenge = Some(challenge);
}

#[when(expr = "the user completes the vocabulary challenge")]
async fn user_completes_vocabulary_challenge(world: &mut BddWorld) {
    if let Some(challenge) = &mut world.challenge {
        challenge.start();
        challenge.challenge_result = ChallengeResult::Vocabulary;
        challenge.update_end_time();
    }
}

// ============================================================================
// Multiple Choice Challenge Steps
// ============================================================================

#[given(expr = "a user starts a multiple choice challenge with {int} questions")]
async fn user_starts_multiple_choice_challenge(world: &mut BddWorld, question_count: usize) {
    let mut questions = Vec::new();
    let options = vec![
        MultipleChoiceOption {
            id: 0,
            name: "Correct".to_string(),
        },
        MultipleChoiceOption {
            id: 1,
            name: "Wrong".to_string(),
        },
    ];

    for i in 0..question_count {
        questions.push(Question {
            question: format!("Question {}", i + 1),
            help: format!("Help {}", i + 1),
            image: None,
            option: 0, // Correct answer is always option 0
        });
    }

    let mc = MultipleChoice {
        id: "test-mc".to_string(),
        name: "Test Multiple Choice".to_string(),
        lang: "en".to_string(),
        options,
        questions,
    };

    world.challenge_type = ChallengeType::MultipleChoice(mc);
    let challenge = Challenge::new(&world.challenge_type, &ChallengeConfig::default());
    world.challenge = Some(challenge);
}

#[when(expr = "the user answers {int} questions correctly and {int} incorrectly")]
async fn user_answers_questions_correctly_and_incorrectly(
    world: &mut BddWorld,
    correct_count: usize,
    incorrect_count: usize,
) {
    if let Some(challenge) = &mut world.challenge {
        challenge.start();

        // Answer correctly
        for i in 0..correct_count {
            let _ = challenge.solve(
                ChallengeInput::MultipleChoice(MultipleChoiceOption {
                    id: 0,
                    name: "Correct".to_string(),
                }),
                i,
            );
        }

        // Answer incorrectly
        for i in correct_count..(correct_count + incorrect_count) {
            let _ = challenge.solve(
                ChallengeInput::MultipleChoice(MultipleChoiceOption {
                    id: 1,
                    name: "Wrong".to_string(),
                }),
                i,
            );
        }

        challenge.update_end_time();
    }
}

// ============================================================================
// Informative Challenge Steps
// ============================================================================

#[given(expr = "a user starts an informative challenge")]
async fn user_starts_informative_challenge(world: &mut BddWorld) {
    let informative = Informative::default();
    world.challenge_type = ChallengeType::Informative(informative);
    let challenge = Challenge::new(&world.challenge_type, &ChallengeConfig::default());
    world.challenge = Some(challenge);
}

#[when(expr = "the user completes the informative challenge")]
async fn user_completes_informative_challenge(world: &mut BddWorld) {
    if let Some(challenge) = &mut world.challenge {
        challenge.start();
        challenge.challenge_result = ChallengeResult::Informative;
        challenge.update_end_time();
    }
}

// ============================================================================
// Generic Challenge Steps
// ============================================================================

#[given(expr = "a user starts a challenge")]
async fn user_starts_challenge(world: &mut BddWorld) {
    user_starts_multiple_choice_challenge(world, 5).await;
}

#[when(expr = "the user abandons the challenge without answers")]
async fn user_abandons_challenge(world: &mut BddWorld) {
    if let Some(challenge) = &mut world.challenge {
        challenge.start();
        challenge.update_end_time();
        // Don't add any answers
    }
}

// ============================================================================
// Best Performance Tracking Steps
// ============================================================================

#[given(expr = "a user completes a challenge with {int}% performance")]
async fn user_completes_challenge_with_performance(world: &mut BddWorld, performance: u32) {
    user_starts_multiple_choice_challenge(world, 10).await;

    let correct = (10 * performance / 100) as usize;
    let incorrect = 10 - correct;

    user_answers_questions_correctly_and_incorrectly(world, correct, incorrect).await;

    let challenge = world.challenge.take().expect("Challenge should exist");
    world.game.challenge_history.add_challenge(challenge);
}

#[when(expr = "the user completes the same challenge with {int}% performance")]
async fn user_completes_same_challenge_with_performance(world: &mut BddWorld, performance: u32) {
    // Use the same challenge config as before
    user_starts_multiple_choice_challenge(world, 10).await;

    let correct = (10 * performance / 100) as usize;
    let incorrect = 10 - correct;

    user_answers_questions_correctly_and_incorrectly(world, correct, incorrect).await;

    let challenge = world.challenge.take().expect("Challenge should exist");
    world.game.challenge_history.add_challenge(challenge);
}

#[given(expr = "a user completes a challenge with {int}% performance in {int} seconds")]
async fn user_completes_challenge_with_performance_and_time(
    world: &mut BddWorld,
    performance: u32,
    seconds: i64,
) {
    user_starts_multiple_choice_challenge(world, 10).await;

    let correct = (10 * performance / 100) as usize;
    let incorrect = 10 - correct;

    if let Some(challenge) = &mut world.challenge {
        challenge.start();

        // Answer correctly
        for i in 0..correct {
            let _ = challenge.solve(
                ChallengeInput::MultipleChoice(MultipleChoiceOption {
                    id: 0,
                    name: "Correct".to_string(),
                }),
                i,
            );
        }

        // Answer incorrectly
        for i in correct..(correct + incorrect) {
            let _ = challenge.solve(
                ChallengeInput::MultipleChoice(MultipleChoiceOption {
                    id: 1,
                    name: "Wrong".to_string(),
                }),
                i,
            );
        }

        // Set specific time
        use chrono::Utc;
        let start = Utc::now();
        let end = start + chrono::Duration::seconds(seconds);
        challenge.start_time = Some(start);
        challenge.end_time = Some(end);
    }

    let challenge = world.challenge.take().expect("Challenge should exist");
    world.game.challenge_history.add_challenge(challenge);
}

#[when(expr = "the user completes the same challenge with {int}% performance in {int} seconds")]
async fn user_completes_same_challenge_with_performance_and_time(
    world: &mut BddWorld,
    performance: u32,
    seconds: i64,
) {
    user_completes_challenge_with_performance_and_time(world, performance, seconds).await;
}

// ============================================================================
// Multiple Challenge Completion Steps (COMPLETE FIX)
// ============================================================================

#[given(expr = "a user completes challenge A with {int}% performance")]
async fn user_completes_challenge_a_with_performance(world: &mut BddWorld, performance: u32) {
    user_starts_multiple_choice_challenge(world, 10).await;

    if let Some(challenge) = &mut world.challenge {
        challenge.challenge_config.id = "challenge-a".to_string();
    }

    let correct = (10 * performance / 100) as usize;
    let incorrect = 10 - correct;

    user_answers_questions_correctly_and_incorrectly(world, correct, incorrect).await;

    let challenge = world.challenge.take().expect("Challenge should exist");
    world.game.challenge_history.add_challenge(challenge);
}

#[given(expr = "the user completes challenge A with {int}% performance")]
async fn user_completes_challenge_a_again_with_performance(world: &mut BddWorld, performance: u32) {
    // This handles subsequent attempts at challenge A
    user_completes_challenge_a_with_performance(world, performance).await;
}

#[given(expr = "the user completes challenge B with {int}% performance")]
async fn user_completes_challenge_b_with_performance(world: &mut BddWorld, performance: u32) {
    user_starts_multiple_choice_challenge(world, 10).await;

    if let Some(challenge) = &mut world.challenge {
        challenge.challenge_config.id = "challenge-b".to_string();
    }

    let correct = (10 * performance / 100) as usize;
    let incorrect = 10 - correct;

    user_answers_questions_correctly_and_incorrectly(world, correct, incorrect).await;

    let challenge = world.challenge.take().expect("Challenge should exist");
    world.game.challenge_history.add_challenge(challenge);
}

#[given(expr = "a user completes a vocabulary challenge with {int}% performance")]
async fn user_completes_vocabulary_with_performance(world: &mut BddWorld, _performance: u32) {
    user_starts_vocabulary_challenge(world).await;

    if let Some(challenge) = &mut world.challenge {
        challenge.challenge_config.id = "vocab-challenge".to_string();
    }

    user_completes_vocabulary_challenge(world).await;

    let challenge = world.challenge.take().expect("Challenge should exist");
    world.game.challenge_history.add_challenge(challenge);
}

#[given(expr = "the user completes a multiple choice challenge with {int}% performance")]
async fn user_completes_a_multiple_choice_with_performance(world: &mut BddWorld, performance: u32) {
    user_starts_multiple_choice_challenge(world, 10).await;

    if let Some(challenge) = &mut world.challenge {
        challenge.challenge_config.id = "mc-challenge".to_string();
    }

    let correct = (10 * performance / 100) as usize;
    let incorrect = 10 - correct;

    user_answers_questions_correctly_and_incorrectly(world, correct, incorrect).await;

    let challenge = world.challenge.take().expect("Challenge should exist");
    world.game.challenge_history.add_challenge(challenge);
}

#[given(expr = "the user completes an informative challenge with {int}% performance")]
async fn user_completes_an_informative_with_performance(world: &mut BddWorld, _performance: u32) {
    user_starts_informative_challenge(world).await;

    if let Some(challenge) = &mut world.challenge {
        challenge.challenge_config.id = "info-challenge".to_string();
    }

    user_completes_informative_challenge(world).await;

    let challenge = world.challenge.take().expect("Challenge should exist");
    world.game.challenge_history.add_challenge(challenge);
}

// ============================================================================
// Contextual Choice Challenge Steps
// ============================================================================

#[given(expr = "a contextual choice challenge with {int} items")]
async fn contextual_choice_challenge_with_n_items(world: &mut BddWorld, item_count: usize) {
    let mut items = Vec::new();

    for i in 0..item_count {
        items.push(ContextItem {
            template: format!("Item {} with {{0}} choice", i),
            choices: vec![Choice {
                id: 0,
                options: vec!["correct".to_string(), "wrong".to_string()],
                correct_answer: "correct".to_string(),
            }],
        });
    }

    let contextual_choice = ContextualChoice {
        id: "test-cc".to_string(),
        name: "Test Contextual Choice".to_string(),
        description: "Test".to_string(),
        items,
    };

    world.challenge_type = ChallengeType::ContextualChoice(contextual_choice);
    let challenge = Challenge::new(&world.challenge_type, &ChallengeConfig::default());
    world.challenge = Some(challenge);
}

#[when(expr = "the user answers {int} items correctly and {int} incorrectly")]
async fn user_answers_items_correctly_and_incorrectly(
    world: &mut BddWorld,
    correct_count: usize,
    incorrect_count: usize,
) {
    if let Some(challenge) = &mut world.challenge {
        challenge.start();

        let mut results = Vec::new();

        for _ in 0..correct_count {
            results.push(ContextItemChoiceAnswers { ids: vec![0] }); // correct
        }

        for _ in 0..incorrect_count {
            results.push(ContextItemChoiceAnswers { ids: vec![1] }); // incorrect
        }

        challenge.challenge_result = ChallengeResult::ContextualChoice(results);
        challenge.update_end_time();
    }
}

// ============================================================================
// Ordering Challenge Steps
// ============================================================================

#[given(expr = "an ordering challenge with {int} items")]
async fn ordering_challenge_with_n_items(world: &mut BddWorld, item_count: usize) {
    let mut items = Vec::new();

    for i in 0..item_count {
        let elements: Vec<String> = (0..3).map(|j| format!("Element {}-{}", i, j)).collect();
        items.push(OrderingItem {
            elements,
            correct_order: vec![0, 1, 2],
        });
    }

    let ordering = Ordering {
        id: "test-ordering".to_string(),
        name: "Test Ordering".to_string(),
        description: "Test".to_string(),
        items,
    };

    world.challenge_type = ChallengeType::Ordering(ordering);
    let challenge = Challenge::new(&world.challenge_type, &ChallengeConfig::default());
    world.challenge = Some(challenge);
}

#[when(expr = "the user orders {int} items correctly and {int} incorrectly")]
async fn user_orders_items_correctly_and_incorrectly(
    world: &mut BddWorld,
    correct_count: usize,
    incorrect_count: usize,
) {
    if let Some(challenge) = &mut world.challenge {
        challenge.start();

        let mut results = Vec::new();

        for _ in 0..correct_count {
            results.push(OrderingResult {
                order: vec![0, 1, 2],
            });
        }

        for _ in 0..incorrect_count {
            results.push(OrderingResult {
                order: vec![2, 1, 0], // wrong order
            });
        }

        challenge.challenge_result = ChallengeResult::Ordering(results);
        challenge.update_end_time();
    }
}

// ============================================================================
// SortTable Challenge Steps
// ============================================================================

#[given(expr = "a sort table challenge with {int} rows")]
async fn sort_table_challenge_with_n_rows(world: &mut BddWorld, row_count: usize) {
    let columns = vec![SortTableColumn {
        id: "col1".to_string(),
        title: "Column 1".to_string(),
        description: "Test column".to_string(),
    }];

    let mut rows = Vec::new();
    for i in 0..row_count {
        rows.push(SortTableRow {
            id: i,
            values: vec![format!("value-{}", i)],
        });
    }

    let sort_table = SortTable {
        id: "test-sorttable".to_string(),
        name: "Test Sort Table".to_string(),
        description: "Test".to_string(),
        columns,
        rows,
    };

    world.challenge_type = ChallengeType::SortTable(sort_table);
    let challenge = Challenge::new(&world.challenge_type, &ChallengeConfig::default());
    world.challenge = Some(challenge);
}

#[when(expr = "the user sorts {int} rows correctly and {int} incorrectly")]
async fn user_sorts_rows_correctly_and_incorrectly(
    world: &mut BddWorld,
    correct_count: usize,
    incorrect_count: usize,
) {
    if let Some(challenge) = &mut world.challenge {
        challenge.start();

        let mut results = Vec::new();

        for i in 0..correct_count {
            results.push(SortTableRow {
                id: i,
                values: vec![format!("value-{}", i)],
            });
        }

        for i in correct_count..(correct_count + incorrect_count) {
            results.push(SortTableRow {
                id: i,
                values: vec![format!("wrong-value-{}", i)],
            });
        }

        challenge.challenge_result = ChallengeResult::SortTable(results);
        challenge.update_end_time();
    }
}

// ============================================================================
// Performance Assertion Steps
// ============================================================================

#[then(expr = "the challenge should show {int}% performance")]
async fn challenge_should_show_performance(world: &mut BddWorld, expected_performance: u32) {
    let challenge = world.challenge.as_ref().expect("Challenge should exist");
    let actual_performance = challenge.performance(&challenge.challenge_result);
    assert_eq!(
        actual_performance, expected_performance,
        "Expected {}% performance, but got {}%",
        expected_performance, actual_performance
    );
}

#[then(expr = "the performance record should show {int}% for that challenge")]
async fn performance_record_should_show_percentage(world: &mut BddWorld, expected_percentage: u32) {
    // Check if there's a current challenge that hasn't been added to history yet
    if let Some(challenge) = world.challenge.take() {
        // Add it to history so we can check the performance record
        world.game.challenge_history.add_challenge(challenge);
    }

    let performance_record = PerformanceRecord::new_from_history(
        "test-path".to_string(),
        "test-user".to_string(),
        world.game.challenge_history.len(),
        world.game.challenge_history.clone(),
    );

    assert!(
        !performance_record.challenges_performance.is_empty(),
        "Performance record should have at least one challenge. Challenge history has {} challenges.",
        world.game.challenge_history.len()
    );

    let (challenge_id, perf, _time) = &performance_record.challenges_performance[0];

    assert_eq!(
        *perf as u32, expected_percentage,
        "Expected challenge '{}' to show {}% performance, but got {}%",
        challenge_id, expected_percentage, *perf
    );
}

#[then(expr = "the performance record should have {int} unique challenge")]
#[then(expr = "the performance record should have {int} unique challenges")]
async fn performance_record_should_have_unique_challenges(
    world: &mut BddWorld,
    expected_count: usize,
) {
    // Add current challenge to history if needed
    if let Some(challenge) = world.challenge.take() {
        world.game.challenge_history.add_challenge(challenge);
    }

    let performance_record = PerformanceRecord::new_from_history(
        "test-path".to_string(),
        "test-user".to_string(),
        world.game.challenge_history.len(),
        world.game.challenge_history.clone(),
    );

    assert_eq!(
        performance_record.challenges_performance.len(),
        expected_count,
        "Expected {} unique challenge(s), but got {}. Challenges: {:?}",
        expected_count,
        performance_record.challenges_performance.len(),
        performance_record
            .challenges_performance
            .iter()
            .map(|(id, p, t)| (id.as_str(), p, t))
            .collect::<Vec<_>>()
    );
}

#[then(expr = "the performance record should show {int} seconds for that challenge")]
async fn performance_record_should_show_time(world: &mut BddWorld, expected_seconds: i64) {
    // Add current challenge to history if needed
    if let Some(challenge) = world.challenge.take() {
        world.game.challenge_history.add_challenge(challenge);
    }

    let performance_record = PerformanceRecord::new_from_history(
        "test-path".to_string(),
        "test-user".to_string(),
        world.game.challenge_history.len(),
        world.game.challenge_history.clone(),
    );

    assert!(
        !performance_record.challenges_performance.is_empty(),
        "Performance record should have at least one challenge"
    );

    let (_id, _perf, time_ms) = &performance_record.challenges_performance[0];
    let actual_seconds = (*time_ms as i64) / 1000;

    assert_eq!(
        actual_seconds, expected_seconds,
        "Expected {} seconds, but got {} seconds ({} ms)",
        expected_seconds, actual_seconds, time_ms
    );
}
#[then(expr = "the leaderboard should show {int}% for that challenge")]
async fn leaderboard_should_show_percentage(world: &mut BddWorld, expected_percentage: u32) {
    // Leaderboard uses PerformanceRecord
    performance_record_should_show_percentage(world, expected_percentage).await;
}

// ============================================================================
// Overall Performance Calculation Steps
// ============================================================================

#[when(expr = "the performance record is calculated")]
async fn performance_record_is_calculated(world: &mut BddWorld) {
    let performance_record = PerformanceRecord::new_from_history(
        "test-path".to_string(),
        "test-user".to_string(),
        world.game.challenge_history.len(),
        world.game.challenge_history.clone(),
    );

    // Store it in the world for later assertions
    world.achievement_notification = Some(konnektoren_rs::achievements::AchievementDefinition {
        id: format!(
            "perf_record:{}",
            performance_record.challenges_performance.len()
        ),
        name: format!("avg:{}", performance_record.performance_percentage),
        description: "performance_record".to_string(),
        condition: "true".to_string(),
        icon: "📊".to_string(),
    });
}

#[then(expr = "the overall performance should be {int}%")]
async fn overall_performance_should_be(world: &mut BddWorld, expected: u32) {
    let performance_record = PerformanceRecord::new_from_history(
        "test-path".to_string(),
        "test-user".to_string(),
        world.game.challenge_history.len(),
        world.game.challenge_history.clone(),
    );

    assert_eq!(
        performance_record.performance_percentage as u32, expected,
        "Expected overall performance {}%, but got {}%",
        expected, performance_record.performance_percentage
    );
}
