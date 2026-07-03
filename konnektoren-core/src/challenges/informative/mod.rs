#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct Informative {
    /// Unique identifier for the informative content
    pub id: String,
    /// Display name of the content
    pub name: String,
    /// Description of the content
    pub description: String,
    /// Multilingual text content
    pub text: Vec<InformativeText>,
}

impl Default for Informative {
    fn default() -> Self {
        let data = include_str!("../../../../assets/personal_pronouns_info.yml");
        serde_yaml::from_str(data).unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct InformativeText {
    /// Language code
    pub language: String,
    /// The informative text content
    pub text: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_yaml() {
        let yaml_data = include_str!("../../../../assets/personal_pronouns_info.yml");
        let dataset: Informative = serde_yaml::from_str(yaml_data).unwrap();
        assert_eq!(dataset.id, "personal_pronouns_info");
        assert_eq!(dataset.name, "Personal Pronouns Info");
    }

    #[test]
    fn default() {
        let dataset = Informative::default();
        assert_eq!(dataset.id, "personal_pronouns_info");
        assert_eq!(dataset.name, "Personal Pronouns Info");
    }
}
