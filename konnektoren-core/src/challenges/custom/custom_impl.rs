#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct Custom {
    /// Unique identifier for the custom challenge
    pub id: String,
    /// Display name of the challenge
    pub name: String,
    /// Description of the challenge
    pub description: String,
    /// HTML content for the challenge
    pub html: String,
    /// Optional HTML content for results display
    pub results_html: Option<String>,
    /// CSS styles for the challenge
    pub css: String,
    /// JavaScript code for the challenge
    pub js: String,
    /// Optional internationalization data
    pub i18n: Option<String>,
    /// Custom data for the challenge
    pub data: serde_json::Value,
    /// Optional task IDs for selection
    pub task_ids: Option<Vec<usize>>,
    /// Optional package URL for external content
    pub package_url: Option<String>,
}

impl Default for Custom {
    fn default() -> Self {
        crate::assets::default_asset("custom_default.yml").unwrap_or_else(|| Self {
            id: String::new(),
            name: String::new(),
            description: String::new(),
            html: String::new(),
            results_html: None,
            css: String::new(),
            js: String::new(),
            i18n: None,
            data: serde_json::Value::Null,
            task_ids: None,
            package_url: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_dataset() {
        let id = "123".to_string();
        let name = "Test".to_string();
        let data = serde_json::json!({
            "key": "value"
        });
        let dataset = Custom {
            id: id.clone(),
            name: name.clone(),
            description: "".to_string(),
            html: "".to_string(),
            results_html: None,
            css: "".to_string(),
            js: "".to_string(),
            i18n: None,
            data: data.clone(),
            task_ids: None,
            package_url: None,
        };

        assert_eq!(dataset.id, id);
        assert_eq!(dataset.name, name);
        assert_eq!(dataset.data, data);
    }

    #[test]
    fn serialize_dataset() {
        let json_str = r#"{"id":"123","name":"Test","description":"","html":"","results_html":null,"css":"","js":"","i18n":null,"data":{"key":"value"},"task_ids":null,"package_url":null}"#;
        let dataset = Custom {
            id: "123".to_string(),
            name: "Test".to_string(),
            description: "".to_string(),
            html: "".to_string(),
            results_html: None,
            css: "".to_string(),
            js: "".to_string(),
            i18n: None,
            data: serde_json::json!({
                "key": "value"
            }),
            task_ids: None,
            package_url: None,
        };

        let serialized = serde_json::to_string(&dataset).unwrap();
        assert_eq!(serialized, json_str);
    }
}
