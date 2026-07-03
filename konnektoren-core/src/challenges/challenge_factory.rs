use crate::challenges::Base64Serializable;
use crate::challenges::challenge::Challenge;
use crate::challenges::challenge_config::ChallengeConfig;
use crate::challenges::challenge_type::ChallengeType;
use crate::challenges::error::{ChallengeError, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChallengeFactory {
    pub challenge_types: Vec<ChallengeType>,
}

impl Default for ChallengeFactory {
    fn default() -> Self {
        ChallengeFactory {
            challenge_types: vec![
                ChallengeType::default(),
                serde_yaml::from_str(include_str!("../../../assets/articles-1.yml")).unwrap(),
                serde_yaml::from_str(include_str!("../../../assets/past-tense.yml")).unwrap(),
                serde_yaml::from_str(include_str!("../../../assets/sentence_structure.yml"))
                    .unwrap(),
                serde_yaml::from_str(include_str!("../../../assets/dialog_begruessung.yml"))
                    .unwrap(),
            ],
        }
    }
}

impl ChallengeFactory {
    pub fn new() -> Self {
        ChallengeFactory {
            challenge_types: vec![],
        }
    }

    pub fn create_challenge(&self, challenge_config: &ChallengeConfig) -> Result<Challenge> {
        let challenge_type = self
            .challenge_types
            .iter()
            .find(|challenge_type| challenge_type.id() == challenge_config.challenge)
            .ok_or(ChallengeError::ChallengeTypeNotFound)?;
        Ok(Challenge::new(
            &challenge_type.of_tasks(&challenge_config.tasks),
            challenge_config,
        ))
    }

    pub fn add_challenge_from_base64(&mut self, base64_data: &str) -> Result<()> {
        let challenge_type = ChallengeType::from_base64(base64_data)?;
        self.challenge_types.push(challenge_type);
        Ok(())
    }

    pub fn export_challenge_to_base64(&self, challenge_id: &str) -> Result<String> {
        let challenge_type = self
            .challenge_types
            .iter()
            .find(|ct| ct.id() == challenge_id)
            .ok_or_else(|| ChallengeError::ChallengeNotFound(challenge_id.to_string()))?;

        challenge_type.to_base64()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_challenge() {
        let mut challenge_factory = ChallengeFactory::new();
        let challenge_type = ChallengeType::default();
        challenge_factory.challenge_types.push(challenge_type);
        let challenge_config = ChallengeConfig {
            challenge: "konnektoren".to_string(),
            tasks: 2.into(),
            ..Default::default()
        };

        let challenge = challenge_factory.create_challenge(&challenge_config);
        match challenge.unwrap().challenge_type {
            ChallengeType::MultipleChoice(dataset) => {
                assert_eq!(dataset.questions.len(), 2);
            }
            _ => panic!("Invalid challenge type"),
        }
    }

    #[test]
    fn test_add_challenge_from_base64() {
        let mut factory = ChallengeFactory::new();
        let original_challenge = ChallengeType::default();

        // Export to base64
        let base64_data = original_challenge.to_base64().unwrap();

        // Import from base64
        factory.add_challenge_from_base64(&base64_data).unwrap();

        assert_eq!(factory.challenge_types.len(), 1);
        assert_eq!(factory.challenge_types[0].id(), original_challenge.id());
    }

    #[test]
    fn test_export_challenge_to_base64() {
        let factory = ChallengeFactory::default();
        let base64_data = factory.export_challenge_to_base64("konnektoren").unwrap();
        assert!(!base64_data.is_empty());
    }

    #[test]
    fn dialog_observer_challenge_can_be_created() {
        use crate::challenges::ChallengeVariant;
        let factory = ChallengeFactory::default();
        let config = ChallengeConfig {
            challenge: "dialog_begruessung".to_string(),
            variant: Some(ChallengeVariant::DialogObserver),
            tasks: 1.into(),
            ..ChallengeConfig::default()
        };
        let challenge = factory.create_challenge(&config).unwrap();
        assert!(matches!(
            challenge.challenge_type,
            crate::challenges::ChallengeType::Dialog(_)
        ));
        assert_eq!(
            challenge.challenge_config.variant,
            Some(ChallengeVariant::DialogObserver)
        );
    }
}
