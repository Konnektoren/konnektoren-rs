use super::message::Message;
use serde::{Deserialize, Serialize};

/// Top-level container matching `inbox.yml`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct InboxData {
    pub messages: Vec<Message>,
}

#[cfg(test)]
mod tests {
    use super::*;

    const INBOX_YAML: &str = include_str!("../../../assets/inbox.yml");

    fn load() -> InboxData {
        serde_yaml::from_str(INBOX_YAML).expect("Failed to parse inbox YAML")
    }

    #[test]
    fn test_load_inbox_data() {
        assert!(!load().messages.is_empty(), "messages should not be empty");
    }

    #[test]
    fn test_message_structure() {
        use chrono::Datelike;

        let inbox_data = load();
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
        for message in &load().messages {
            assert!(!message.sender.is_empty(), "sender should not be empty");
            assert!(!message.content.is_empty(), "content should not be empty");
        }
    }

    #[test]
    fn test_serde_round_trip() {
        let inbox_data = load();
        let json = serde_json::to_string(&inbox_data).unwrap();
        let back: InboxData = serde_json::from_str(&json).unwrap();
        assert_eq!(inbox_data, back);
    }
}
