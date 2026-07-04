use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Message {
    #[serde(default)]
    pub id: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub sender: String,
    pub content: String,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Default)]
pub struct InboxData {
    pub messages: Vec<Message>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;

    const INBOX_YAML: &str = include_str!("../../assets/inbox.yml");

    #[test]
    fn test_load_inbox_data() {
        let inbox_data: InboxData =
            serde_yaml::from_str(INBOX_YAML).expect("Failed to parse inbox YAML");
        assert!(
            !inbox_data.messages.is_empty(),
            "messages should not be empty"
        );
    }

    #[test]
    fn test_message_structure() {
        let inbox_data: InboxData =
            serde_yaml::from_str(INBOX_YAML).expect("Failed to parse inbox YAML");
        let first = &inbox_data.messages[0];

        assert_eq!(
            first.id.as_deref(),
            Some("0e3f15e0-6e2c-4d0c-ad0c-fe6cb8c2f8d0")
        );
        assert_eq!(first.sender, "Konnektoren Team");
        assert!(first.content.contains("Welcome to your Konnektoren inbox"));
        assert_eq!(first.timestamp.year(), 2024);
    }

    #[test]
    fn test_all_messages_have_required_fields() {
        let inbox_data: InboxData =
            serde_yaml::from_str(INBOX_YAML).expect("Failed to parse inbox YAML");

        for message in &inbox_data.messages {
            assert!(!message.sender.is_empty(), "sender should not be empty");
            assert!(!message.content.is_empty(), "content should not be empty");
        }
    }
}
