use cucumber::World;
use konnektoren_rs::core::error::Result;
pub mod steps;
use konnektoren_rs::core::controller::GameController;
use konnektoren_rs::prelude::*;
use std::sync::Arc;

#[derive(Debug, World)]
pub struct BddWorld {
    pub session: Session,
    pub challenge_type: ChallengeType,
    pub challenge: Option<Challenge>,
    pub challenge_result: Option<ChallengeResult>,
    pub game_path: GamePath,
    pub factory: Option<ChallengeFactory>,
    pub last_command_result: Result<()>,
    pub last_solve_correct: Option<bool>,
    pub game: Game,
    pub unlocked_achievements: Vec<AchievementDefinition>,
    pub achievement_notification: Option<AchievementDefinition>,
    pub controller: Option<Arc<GameController>>,
}

impl Default for BddWorld {
    fn default() -> Self {
        let session = Session::new("123".to_string());
        let challenge_type = ChallengeType::default();
        let game_path = GamePath::default();
        let mut factory = ChallengeFactory::new();
        factory.challenge_types.push(challenge_type.clone());
        let game = Game {
            game_paths: vec![game_path.clone()],
            challenge_factory: factory.clone(),
            ..Default::default()
        };

        Self {
            session,
            challenge: None,
            challenge_type,
            challenge_result: None,
            game_path,
            factory: Some(factory),
            last_command_result: Ok(()),
            last_solve_correct: None,
            game,
            unlocked_achievements: Vec::new(),
            achievement_notification: None,
            controller: None,
        }
    }
}

#[tokio::main]
async fn main() {
    // ---- JUnit XML output ----
    #[cfg(feature = "output-junit")]
    {
        let junit_file =
            std::fs::File::create("junit-report.xml").expect("Failed to create JUnit XML file");

        BddWorld::cucumber()
            .max_concurrent_scenarios(1)
            .with_writer(cucumber::writer::JUnit::new(junit_file, 0))
            .run("tests/features")
            .await;
        // No run_and_exit, no summaries – JUnit is just a structured sink.
        return;
    }

    // ---- JSON output ----
    #[cfg(all(feature = "output-json", not(feature = "output-junit")))]
    {
        let json_file = std::fs::File::create("cucumber-report.json")
            .expect("Failed to create JSON output file");

        BddWorld::cucumber()
            .max_concurrent_scenarios(1)
            .with_writer(cucumber::writer::Json::new(json_file))
            .run("tests/features")
            .await;
        // Same here: just write JSON events.
        return;
    }

    // ---- Pretty terminal output (default / output-pretty) ----
    #[cfg(any(all(not(feature = "output-json"), not(feature = "output-junit"))))]
    {
        use cucumber::WriterExt;
        BddWorld::cucumber()
            .max_concurrent_scenarios(1)
            .with_writer(
                cucumber::writer::Basic::raw(
                    std::io::stdout(),
                    cucumber::writer::Coloring::Never,
                    0,
                )
                .summarized()
                .assert_normalized(),
            )
            .run_and_exit("tests/features")
            .await;
    }
}
