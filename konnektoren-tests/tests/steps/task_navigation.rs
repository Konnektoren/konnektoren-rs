use crate::BddWorld;
use cucumber::{given, then, when};
use konnektoren_rs::core::challenges::{
    Solvable,
    contextual_choice::{Choice, ContextItem, ContextItemChoiceAnswers, ContextualChoice},
    task_pattern::TaskPattern,
};
use konnektoren_rs::core::commands::{ChallengeCommand, Command, CommandTrait};
use konnektoren_rs::prelude::*;

#[given(expr = "a konnektoren challenge with range pattern {string} is loaded")]
async fn a_konnektoren_challenge_with_range_pattern(world: &mut BddWorld, pattern: String) {
    load_konnektoren_challenge_with_pattern(world, pattern).await;
}

#[given(expr = "a konnektoren challenge with random pattern {string} is loaded")]
async fn a_konnektoren_challenge_with_random_pattern(world: &mut BddWorld, pattern: String) {
    load_konnektoren_challenge_with_pattern(world, pattern).await;
}

#[given(expr = "a konnektoren challenge with exact pattern {string} is loaded")]
async fn a_konnektoren_challenge_with_exact_pattern(world: &mut BddWorld, pattern: String) {
    load_konnektoren_challenge_with_pattern(world, pattern).await;
}

async fn load_konnektoren_challenge_with_pattern(world: &mut BddWorld, pattern: String) {
    // Parse the task pattern
    let task_pattern = TaskPattern::parse(&pattern).expect("Invalid task pattern");

    // Create challenge config with the pattern
    let challenge_config = ChallengeConfig {
        id: "konnektoren_test".to_string(),
        name: "Konnektoren Test".to_string(),
        description: "Test konnektoren challenge with pattern".to_string(),
        challenge: "konnektoren".to_string(),
        variant: None,
        tasks: task_pattern.clone(),
        unlock_points: 0,
        position: Some((0, 0)),
        icon: None,
    };

    // Load the default konnektoren challenge type
    let konnektoren_challenge = ChallengeType::default();

    // Create factory and add the challenge
    let mut factory = ChallengeFactory::new();
    factory.challenge_types.push(konnektoren_challenge);
    world.factory = Some(factory.clone());

    // Create the challenge with the pattern
    let challenge = factory
        .create_challenge(&challenge_config)
        .expect("Failed to create challenge");

    world.challenge = Some(challenge.clone());
    world.challenge_type = challenge.challenge_type.clone();

    // Update game state challenge
    world.session.game_state.challenge = challenge;
    world.session.game_state.current_task_index = 0;

    // commands will see the correct task count
    let current_game_path_index = world.session.game_state.current_game_path;
    let current_challenge_index = world.session.game_state.current_challenge_index;

    // Replace the challenge config in the game path
    world.session.game_state.game.game_paths[current_game_path_index].challenges
        [current_challenge_index] = challenge_config;
}

#[then(expr = "the current task should be valid")]
async fn the_current_task_should_be_valid(world: &mut BddWorld) {
    let challenge = world.session.game_state.challenge.challenge_type.clone();
    let current_index = world.session.game_state.current_task_index;

    match challenge {
        ChallengeType::MultipleChoice(mc) => {
            assert!(
                current_index < mc.questions.len(),
                "Task index {} should be less than questions length {}",
                current_index,
                mc.questions.len()
            );

            // Verify the question exists and has valid data
            let question = &mc.questions[current_index];
            assert!(
                !question.question.is_empty(),
                "Question text should not be empty"
            );
            assert!(
                !question.help.is_empty(),
                "Question help should not be empty"
            );
            assert!(
                question.option < mc.options.len(),
                "Question option {} should be valid (< {})",
                question.option,
                mc.options.len()
            );
        }
        _ => panic!("Expected MultipleChoice challenge type"),
    }
}

#[then(expr = "the task count should be {int}")]
async fn the_task_count_should_be(world: &mut BddWorld, expected_count: usize) {
    let challenge = &world.session.game_state.challenge;

    match &challenge.challenge_type {
        ChallengeType::MultipleChoice(mc) => {
            assert_eq!(
                mc.questions.len(),
                expected_count,
                "Expected {} tasks, but got {}",
                expected_count,
                mc.questions.len()
            );
        }
        _ => panic!("Expected MultipleChoice challenge type"),
    }
}

#[when(expr = "all tasks in the range are solved")]
async fn all_tasks_in_range_are_solved(world: &mut BddWorld) {
    // Collect all correct options first
    let correct_options: Vec<usize> = match &world.session.game_state.challenge.challenge_type {
        ChallengeType::MultipleChoice(mc) => mc.questions.iter().map(|q| q.option).collect(),
        _ => panic!("Expected MultipleChoice challenge type"),
    };

    // Now solve all tasks using the collected options
    for (i, &correct_option) in correct_options.iter().enumerate() {
        let command = Command::Challenge(ChallengeCommand::SolveOption(correct_option));

        match command.execute(&mut world.session.game_state) {
            Ok(_) => {}
            Err(e) => {
                // If we hit "No more tasks", that's expected at the end
                if !e.to_string().contains("No more tasks") {
                    panic!("Failed to solve task {}: {}", i, e);
                }
            }
        }
    }
}

#[then(expr = "{int} tasks should be completed")]
async fn tasks_should_be_completed(world: &mut BddWorld, expected_count: usize) {
    let challenge = &world.session.game_state.challenge;

    match &challenge.challenge_result {
        ChallengeResult::MultipleChoice(results) => {
            assert_eq!(
                results.len(),
                expected_count,
                "Expected {} completed tasks, but got {}",
                expected_count,
                results.len()
            );
        }
        ChallengeResult::ContextualChoice(results) => {
            assert_eq!(
                results.len(),
                expected_count,
                "Expected {} completed tasks, but got {}",
                expected_count,
                results.len()
            );
        }
        ChallengeResult::GapFill(results) => {
            assert_eq!(
                results.len(),
                expected_count,
                "Expected {} completed tasks, but got {}",
                expected_count,
                results.len()
            );
        }
        ChallengeResult::SortTable(results) => {
            assert_eq!(
                results.len(),
                expected_count,
                "Expected {} completed tasks, but got {}",
                expected_count,
                results.len()
            );
        }
        ChallengeResult::Ordering(results) => {
            assert_eq!(
                results.len(),
                expected_count,
                "Expected {} completed tasks, but got {}",
                expected_count,
                results.len()
            );
        }
        ChallengeResult::Informative => {
            assert_eq!(0, expected_count, "Informative challenges don't have tasks");
        }
        ChallengeResult::Vocabulary => {
            assert_eq!(
                0, expected_count,
                "Vocabulary challenges are always complete"
            );
        }
        ChallengeResult::Custom(_) => {
            // Custom challenges don't track individual task completion in the same way
            assert_eq!(
                0, expected_count,
                "Custom challenges don't have tasks in the standard way"
            );
        }
        ChallengeResult::Dialog(answers) => {
            assert_eq!(
                answers.len(),
                expected_count,
                "Expected {} completed dialog turns, but got {}",
                expected_count,
                answers.len()
            );
        }
    }
}

#[then(expr = "all completed tasks should have valid answers")]
async fn all_completed_tasks_should_have_valid_answers(world: &mut BddWorld) {
    let challenge = &world.session.game_state.challenge;

    match (&challenge.challenge_result, &challenge.challenge_type) {
        (ChallengeResult::MultipleChoice(results), ChallengeType::MultipleChoice(mc)) => {
            assert!(!results.is_empty(), "Should have completed some tasks");

            // Verify all results are valid options
            for (i, result) in results.iter().enumerate() {
                assert!(
                    result.id < mc.options.len(),
                    "Result {} has invalid option id {}",
                    i,
                    result.id
                );
            }
        }
        _ => panic!("Expected MultipleChoice result and challenge type"),
    }
}

#[given(expr = "the current task index is at the last task in range")]
async fn the_current_task_index_is_at_last_task_in_range(world: &mut BddWorld) {
    let challenge = &world.session.game_state.challenge;

    let last_index = match &challenge.challenge_type {
        ChallengeType::MultipleChoice(mc) => mc.questions.len() - 1,
        _ => panic!("Expected MultipleChoice challenge type"),
    };

    world.session.game_state.current_task_index = last_index;
}

#[then(expr = "the first question should be {string}")]
async fn the_first_question_should_be(world: &mut BddWorld, expected_question: String) {
    let challenge = &world.session.game_state.challenge;

    match &challenge.challenge_type {
        ChallengeType::MultipleChoice(mc) => {
            assert!(
                !mc.questions.is_empty(),
                "Challenge should have at least one question"
            );

            let first_question = &mc.questions[0].question;
            assert_eq!(
                first_question, &expected_question,
                "Expected first question to be '{}', but got '{}'",
                expected_question, first_question
            );
        }
        _ => panic!("Expected MultipleChoice challenge type"),
    }
}

#[then(expr = "the second question should be {string}")]
async fn the_second_question_should_be(world: &mut BddWorld, expected_question: String) {
    let challenge = &world.session.game_state.challenge;

    match &challenge.challenge_type {
        ChallengeType::MultipleChoice(mc) => {
            assert!(
                mc.questions.len() >= 2,
                "Challenge should have at least two questions"
            );

            let second_question = &mc.questions[1].question;
            assert_eq!(
                second_question, &expected_question,
                "Expected second question to be '{}', but got '{}'",
                expected_question, second_question
            );
        }
        _ => panic!("Expected MultipleChoice challenge type"),
    }
}

#[then(expr = "the third question should be {string}")]
async fn the_third_question_should_be(world: &mut BddWorld, expected_question: String) {
    let challenge = &world.session.game_state.challenge;

    match &challenge.challenge_type {
        ChallengeType::MultipleChoice(mc) => {
            assert!(
                mc.questions.len() >= 3,
                "Challenge should have at least three questions"
            );

            let third_question = &mc.questions[2].question;
            assert_eq!(
                third_question, &expected_question,
                "Expected third question to be '{}', but got '{}'",
                expected_question, third_question
            );
        }
        _ => panic!("Expected MultipleChoice challenge type"),
    }
}

#[given(expr = "the current question is {string}")]
#[then(expr = "the current question is {string}")]
async fn the_current_question_is(world: &mut BddWorld, expected_question: String) {
    let challenge = &world.session.game_state.challenge;
    let current_index = world.session.game_state.current_task_index;

    match &challenge.challenge_type {
        ChallengeType::MultipleChoice(mc) => {
            assert!(
                current_index < mc.questions.len(),
                "Current task index {} should be less than questions length {}",
                current_index,
                mc.questions.len()
            );

            let current_question = &mc.questions[current_index].question;
            assert_eq!(
                current_question, &expected_question,
                "Expected current question to be '{}', but got '{}'",
                expected_question, current_question
            );
        }
        _ => panic!("Expected MultipleChoice challenge type"),
    }
}

#[when(expr = "the task is solved correctly")]
async fn the_task_is_solved_correctly(world: &mut BddWorld) {
    // Get the correct option for the current task
    let correct_option = match &world.session.game_state.challenge.challenge_type {
        ChallengeType::MultipleChoice(mc) => {
            let current_index = world.session.game_state.current_task_index;
            mc.questions[current_index].option
        }
        _ => panic!("Expected MultipleChoice challenge type"),
    };

    let command = Command::Challenge(ChallengeCommand::SolveOption(correct_option));

    match command.execute(&mut world.session.game_state) {
        Ok(_) => {}
        Err(e) => {
            panic!("Failed to solve task correctly: {}", e);
        }
    }
}

#[when(expr = "task {int} is solved correctly")]
async fn task_n_is_solved_correctly(world: &mut BddWorld, task_index: usize) {
    // First navigate to the specified task
    world.session.game_state.current_task_index = task_index;

    // Then solve it correctly
    the_task_is_solved_correctly(world).await;
}

#[then(expr = "task {int} should be answered")]
async fn task_n_should_be_answered(world: &mut BddWorld, task_index: usize) {
    let challenge = &world.session.game_state.challenge;

    match &challenge.challenge_result {
        ChallengeResult::MultipleChoice(results) => {
            // Check if we have an answer for this task index
            assert!(
                task_index < results.len(),
                "Task {} should be answered, but only {} tasks have been answered",
                task_index,
                results.len()
            );

            // Verify the answer is valid
            let answer = &results[task_index];
            match &challenge.challenge_type {
                ChallengeType::MultipleChoice(mc) => {
                    assert!(
                        answer.id < mc.options.len(),
                        "Answer for task {} has invalid option id {}",
                        task_index,
                        answer.id
                    );
                }
                _ => panic!("Expected MultipleChoice challenge type"),
            }
        }
        _ => panic!("Expected MultipleChoice result"),
    }
}

#[then(expr = "the task should show as already answered")]
async fn the_task_should_show_as_already_answered(world: &mut BddWorld) {
    let current_index = world.session.game_state.current_task_index;
    task_n_should_be_answered(world, current_index).await;
}

#[when(expr = "the task is solved incorrectly")]
async fn the_task_is_solved_incorrectly(world: &mut BddWorld) {
    // Get a wrong option for the current task
    let (wrong_option, question_text) = match &world.session.game_state.challenge.challenge_type {
        ChallengeType::MultipleChoice(mc) => {
            let current_index = world.session.game_state.current_task_index;
            let question = &mc.questions[current_index];
            let correct_option = question.option;

            // Find a different option (wrong one)
            let wrong_option = mc
                .options
                .iter()
                .find(|opt| opt.id != correct_option)
                .expect("Should have at least one wrong option")
                .id;

            (wrong_option, question.question.clone())
        }
        _ => panic!("Expected MultipleChoice challenge type"),
    };

    tracing::debug!(
        "Solving '{}' incorrectly with option {}",
        question_text,
        wrong_option
    );

    let command = Command::Challenge(ChallengeCommand::SolveOption(wrong_option));

    match command.execute(&mut world.session.game_state) {
        Ok(_) => {}
        Err(e) => {
            panic!("Failed to solve task incorrectly: {}", e);
        }
    }
}

#[then(expr = "task {int} should be answered correctly")]
async fn task_n_should_be_answered_correctly(world: &mut BddWorld, task_index: usize) {
    let challenge = &world.session.game_state.challenge;

    match (&challenge.challenge_result, &challenge.challenge_type) {
        (ChallengeResult::MultipleChoice(results), ChallengeType::MultipleChoice(mc)) => {
            assert!(
                task_index < results.len(),
                "Task {} should be answered, but only {} tasks have been answered",
                task_index,
                results.len()
            );

            let answer = &results[task_index];
            let question = &mc.questions[task_index];

            assert_eq!(
                answer.id, question.option,
                "Task {} should be answered correctly. Expected option {}, but got {}",
                task_index, question.option, answer.id
            );
        }
        _ => panic!("Expected MultipleChoice challenge type and result"),
    }
}

#[then(expr = "task {int} should be answered incorrectly")]
async fn task_n_should_be_answered_incorrectly(world: &mut BddWorld, task_index: usize) {
    let challenge = &world.session.game_state.challenge;

    match (&challenge.challenge_result, &challenge.challenge_type) {
        (ChallengeResult::MultipleChoice(results), ChallengeType::MultipleChoice(mc)) => {
            assert!(
                task_index < results.len(),
                "Task {} should be answered, but only {} tasks have been answered",
                task_index,
                results.len()
            );

            let answer = &results[task_index];
            let question = &mc.questions[task_index];

            assert_ne!(
                answer.id, question.option,
                "Task {} should be answered incorrectly. Both answer and correct option are {}",
                task_index, answer.id
            );
        }
        _ => panic!("Expected MultipleChoice challenge type and result"),
    }
}

#[then(expr = "the challenge performance should be {int}")]
async fn the_challenge_performance_should_be(world: &mut BddWorld, expected_performance: u32) {
    let challenge = &world.session.game_state.challenge;
    let actual_performance = challenge.performance(&challenge.challenge_result);

    assert_eq!(
        actual_performance, expected_performance,
        "Expected performance {}%, but got {}%",
        expected_performance, actual_performance
    );
}

#[given(expr = "a contextual choice challenge with {int} items is loaded")]
async fn a_contextual_choice_challenge_with_items(world: &mut BddWorld, item_count: usize) {
    let mut items = Vec::new();

    for i in 0..item_count {
        items.push(ContextItem {
            template: format!("Sentence {{0}} {{1}} example {}", i),
            choices: vec![
                Choice {
                    id: 0,
                    options: vec!["option1".to_string(), "option2".to_string()],
                    correct_answer: "option1".to_string(),
                },
                Choice {
                    id: 1,
                    options: vec!["optionA".to_string(), "optionB".to_string()],
                    correct_answer: "optionA".to_string(),
                },
            ],
        });
    }

    let contextual_choice = ContextualChoice {
        id: "test-cc".to_string(), // This is the challenge type ID
        name: "Test Contextual Choice".to_string(),
        description: "Test navigation".to_string(),
        items,
    };

    let challenge_type = ChallengeType::ContextualChoice(contextual_choice);

    // The challenge config ID must match the challenge type ID
    let challenge_config = ChallengeConfig {
        id: "contextual_test_config".to_string(),
        name: "Contextual Test".to_string(),
        description: "Test contextual challenge".to_string(),
        challenge: "test-cc".to_string(), // This must match the challenge type ID
        variant: None,
        tasks: item_count.into(),
        unlock_points: 0,
        position: Some((0, 0)),
        icon: None,
    };

    // Create factory and add the challenge type
    let mut factory = ChallengeFactory::new();
    factory.challenge_types.push(challenge_type.clone());

    // IMPORTANT: Update the session's game factory too!
    world.session.game_state.game.challenge_factory = factory.clone();

    // Now create the challenge using the factory
    let challenge = factory
        .create_challenge(&challenge_config)
        .expect("Failed to create challenge");

    world.factory = Some(factory);
    world.challenge = Some(challenge.clone());
    world.challenge_type = challenge.challenge_type.clone();

    // Update game state
    world.session.game_state.challenge = challenge;
    world.session.game_state.current_task_index = 0;

    // Update the challenge config in the game path
    let current_game_path_index = world.session.game_state.current_game_path;
    let current_challenge_index = world.session.game_state.current_challenge_index;

    world.session.game_state.game.game_paths[current_game_path_index].challenges
        [current_challenge_index] = challenge_config;
}

#[when(expr = "the contextual choice task is solved correctly")]
async fn the_contextual_choice_task_is_solved_correctly(world: &mut BddWorld) {
    let current_index = world.session.game_state.current_task_index;

    // Create correct input based on the challenge structure
    let input = ChallengeInput::ContextualChoice(ContextItemChoiceAnswers {
        ids: vec![0, 0], // First option from each choice (both correct)
    });

    // Use the solve method
    match world
        .session
        .game_state
        .challenge
        .solve(input, current_index)
    {
        Ok(is_correct) => {
            assert!(is_correct, "Answer should be correct");

            // The solve method in challenge_command.rs handles incrementing task_index,
            // but we need to use the command for that
            let command = Command::Challenge(ChallengeCommand::NextTask);
            let _ = command.execute(&mut world.session.game_state);
        }
        Err(e) => {
            panic!("Failed to solve contextual choice task: {}", e);
        }
    }
}

#[then(expr = "task {int} should be answered correctly for contextual choice")]
async fn task_n_should_be_answered_correctly_for_contextual_choice(
    world: &mut BddWorld,
    task_index: usize,
) {
    let challenge = &world.session.game_state.challenge;

    match (&challenge.challenge_result, &challenge.challenge_type) {
        (ChallengeResult::ContextualChoice(results), ChallengeType::ContextualChoice(cc)) => {
            assert!(
                task_index < results.len(),
                "Task {} should be answered, but only {} tasks have been answered",
                task_index,
                results.len()
            );

            let answer = &results[task_index];
            let item = &cc.items[task_index];

            assert_eq!(
                answer.ids.len(),
                item.choices.len(),
                "Answer should have same number of choices as item"
            );

            // Verify all choices are correct
            for (i, (choice, &answer_id)) in item.choices.iter().zip(&answer.ids).enumerate() {
                assert!(
                    answer_id < choice.options.len(),
                    "Answer id {} should be valid for choice {} (max {})",
                    answer_id,
                    i,
                    choice.options.len() - 1
                );

                let selected_option = &choice.options[answer_id];
                assert_eq!(
                    selected_option, &choice.correct_answer,
                    "Task {} choice {} should be answered correctly. Selected '{}', expected '{}'",
                    task_index, i, selected_option, choice.correct_answer
                );
            }
        }
        _ => panic!("Expected ContextualChoice challenge type and result"),
    }
}
