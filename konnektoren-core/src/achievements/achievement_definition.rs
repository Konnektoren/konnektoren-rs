use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct AchievementDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub condition: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct AchievementDefinitions {
    pub achievements: Vec<AchievementDefinition>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_achievement_definition() {
        let achievement_definition = AchievementDefinition {
            id: "123".to_string(),
            name: "Konnektoren #1".to_string(),
            description: "Your first Konnektoren challenge!".to_string(),
            icon: "icon.png".to_string(),
            condition: "konnektoren".to_string(),
        };
        assert_eq!(achievement_definition.id, "123");
        assert_eq!(achievement_definition.name, "Konnektoren #1");
        assert_eq!(
            achievement_definition.description,
            "Your first Konnektoren challenge!"
        );
        assert_eq!(achievement_definition.icon, "icon.png");
        assert_eq!(achievement_definition.condition, "konnektoren");
    }

    #[test]
    fn deserialize_achievement_definitions() {
        let data = include_str!("../../../assets/achievements.yml");
        let achievement_definitions: AchievementDefinitions = serde_yaml::from_str(data).unwrap();
        assert!(!achievement_definitions.achievements.is_empty());
    }
}
