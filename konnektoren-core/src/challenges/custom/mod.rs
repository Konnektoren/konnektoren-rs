#[cfg(feature = "js")]
mod konnektoren_js;

use serde::{Deserialize, Serialize};

#[cfg(feature = "js")]
pub use konnektoren_js::KonnektorenJs;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Custom {
    pub id: String,
    pub name: String,
    pub description: String,
    pub html: String,
    pub css: String,
    pub js: String,
    pub data: serde_json::Value,
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
            css: "".to_string(),
            js: "".to_string(),
            data: data.clone(),
        };

        assert_eq!(dataset.id, id);
        assert_eq!(dataset.name, name);
        assert_eq!(dataset.data, data);
    }

    #[test]
    fn serialize_dataset() {
        let json_str = r#"{"id":"123","name":"Test","description":"","html":"","css":"","js":"","data":{"key":"value"}}"#;
        let dataset = Custom {
            id: "123".to_string(),
            name: "Test".to_string(),
            description: "".to_string(),
            html: "".to_string(),
            css: "".to_string(),
            js: "".to_string(),
            data: serde_json::json!({
                "key": "value"
            }),
        };

        let serialized = serde_json::to_string(&dataset).unwrap();
        assert_eq!(serialized, json_str);
    }
}
