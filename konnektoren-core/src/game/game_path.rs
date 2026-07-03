use crate::challenges::challenge_config::ChallengeConfig;
use crate::game::Map;
#[cfg(feature = "schema")]
use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct GamePath {
    pub id: String,
    pub name: String,
    pub challenges: Vec<ChallengeConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub map: Option<Map>,
}

impl Default for GamePath {
    fn default() -> Self {
        let data = include_str!("../../../assets/konnektoren_path.yml");
        serde_yaml::from_str(data).unwrap()
    }
}

impl GamePath {
    pub fn get_challenge_config(&self, challenge_config_id: &str) -> Option<&ChallengeConfig> {
        self.challenges
            .iter()
            .find(|challenge| challenge.id == challenge_config_id)
    }

    pub fn challenge_ids(&self) -> Vec<String> {
        self.challenges
            .iter()
            .map(|challenge| challenge.id.clone())
            .collect()
    }

    pub fn insert_before(&mut self, before_id: &str, config: ChallengeConfig) {
        if let Some(pos) = self.challenges.iter().position(|c| c.id == before_id) {
            self.challenges.insert(pos, config);
        }
    }

    pub fn insert_after(&mut self, after_id: &str, config: ChallengeConfig) {
        if let Some(pos) = self.challenges.iter().position(|c| c.id == after_id) {
            self.challenges.insert(pos + 1, config);
        }
    }

    pub fn next_challenge_id(&self, challenge_id: &str) -> Option<String> {
        let mut iter = self.challenges.iter();
        while let Some(challenge) = iter.next() {
            if challenge.id == challenge_id {
                return iter.next().map(|c| c.id.clone());
            }
        }
        None
    }
}

#[cfg(feature = "schema")]
impl GamePath {
    /// Get the JSON schema for `GamePath` (for documentation / sharing with third parties)
    pub fn schema() -> serde_json::Value {
        let schema = schema_for!(GamePath);
        serde_json::to_value(schema).expect("schemars Schema is always JSON-serializable")
    }

    /// Get the JSON schema as a pretty-printed string
    pub fn schema_json() -> String {
        serde_json::to_string_pretty(&Self::schema())
            .expect("serde_json::Value is always serializable to string")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_game_path() {
        let game_path = GamePath::default();
        assert_eq!(game_path.name, "Konnektoren");
        assert!(!game_path.challenges.is_empty());
    }

    #[test]
    fn get_challenge_config() {
        let game_path = GamePath::default();
        let challenge_config = game_path.get_challenge_config("unknown");
        assert!(challenge_config.is_none());
        let challenge_config = game_path.get_challenge_config("konnektoren-1");
        assert!(challenge_config.is_some());
    }

    #[test]
    fn test_challenge_ids() {
        let game_path = GamePath::default();
        let challenge_ids = game_path.challenge_ids();
        assert_eq!(challenge_ids.len(), game_path.challenges.len());
    }

    #[test]
    fn test_next_challenge_id() {
        let game_path = GamePath::default();
        let next_challenge_id = game_path.next_challenge_id("konnektoren-1");
        assert_eq!(next_challenge_id, Some("konnektoren-2".to_string()));
    }

    fn make_game_path(ids: &[&str]) -> GamePath {
        GamePath {
            id: "test".to_string(),
            name: "Test".to_string(),
            map: None,
            challenges: ids
                .iter()
                .map(|id| ChallengeConfig {
                    id: id.to_string(),
                    ..ChallengeConfig::default()
                })
                .collect(),
        }
    }

    fn challenge_ids(game_path: &GamePath) -> Vec<&str> {
        game_path.challenges.iter().map(|c| c.id.as_str()).collect()
    }

    #[test]
    fn test_insert_before() {
        let mut game_path = make_game_path(&["a", "b", "c"]);
        game_path.insert_before(
            "b",
            ChallengeConfig {
                id: "x".to_string(),
                ..ChallengeConfig::default()
            },
        );
        assert_eq!(challenge_ids(&game_path), vec!["a", "x", "b", "c"]);
    }

    #[test]
    fn test_insert_before_unknown_id_is_noop() {
        let mut game_path = make_game_path(&["a", "b"]);
        game_path.insert_before(
            "z",
            ChallengeConfig {
                id: "x".to_string(),
                ..ChallengeConfig::default()
            },
        );
        assert_eq!(challenge_ids(&game_path), vec!["a", "b"]);
    }

    #[test]
    fn test_insert_after() {
        let mut game_path = make_game_path(&["a", "b", "c"]);
        game_path.insert_after(
            "b",
            ChallengeConfig {
                id: "x".to_string(),
                ..ChallengeConfig::default()
            },
        );
        assert_eq!(challenge_ids(&game_path), vec!["a", "b", "x", "c"]);
    }

    #[test]
    fn test_insert_after_unknown_id_is_noop() {
        let mut game_path = make_game_path(&["a", "b"]);
        game_path.insert_after(
            "z",
            ChallengeConfig {
                id: "x".to_string(),
                ..ChallengeConfig::default()
            },
        );
        assert_eq!(challenge_ids(&game_path), vec!["a", "b"]);
    }

    #[test]
    fn test_game_path_json_round_trip() {
        let game_path = GamePath::default();

        // Serialize to JSON
        let json = serde_json::to_string(&game_path).expect("GamePath should serialize to JSON");
        assert!(!json.is_empty());

        // Deserialize back
        let from_json: GamePath =
            serde_json::from_str(&json).expect("GamePath should deserialize from JSON");
        assert_eq!(game_path, from_json);
    }

    #[cfg(feature = "schema")]
    #[test]
    fn test_game_path_json_schema() {
        let schema = GamePath::schema();
        assert!(schema.is_object());

        let schema_str =
            serde_json::to_string_pretty(&schema).expect("schema should serialize to string");
        // The schema must describe the key fields
        assert!(
            schema_str.contains("challenges"),
            "schema should mention 'challenges'"
        );
        assert!(schema_str.contains("id"), "schema should mention 'id'");

        // schema_json convenience method should produce the same output
        let schema_json = GamePath::schema_json();
        assert!(!schema_json.is_empty());
    }
}
