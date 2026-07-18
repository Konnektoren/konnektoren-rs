#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct SortTable {
    /// Unique identifier for the challenge
    pub id: String,
    /// Display name of the challenge
    pub name: String,
    /// Description of the challenge
    pub description: String,
    /// Table column definitions
    pub columns: Vec<SortTableColumn>,
    /// Table rows to be sorted
    pub rows: Vec<SortTableRow>,
}

impl Default for SortTable {
    fn default() -> Self {
        crate::assets::default_asset("personal_pronouns.yml").unwrap_or_else(|| Self {
            id: String::new(),
            name: String::new(),
            description: String::new(),
            columns: Vec::new(),
            rows: Vec::new(),
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct SortTableColumn {
    /// Column identifier
    pub id: String,
    /// Column title
    pub title: String,
    /// Column description
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct SortTableRow {
    /// Row identifier
    pub id: usize,
    /// Values for each column
    pub values: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_dataset() {
        let id = "123".to_string();
        let name = "Test".to_string();
        let description = "Test".to_string();
        let columns = vec![
            SortTableColumn {
                id: "column1".to_string(),
                title: "Column 1".to_string(),
                description: "Description 1".to_string(),
            },
            SortTableColumn {
                id: "column2".to_string(),
                title: "Column 2".to_string(),
                description: "Description 2".to_string(),
            },
        ];
        let rows = vec![
            SortTableRow {
                id: 1,
                values: vec!["Value 1".to_string(), "Value 2".to_string()],
            },
            SortTableRow {
                id: 2,
                values: vec!["Value 3".to_string(), "Value 4".to_string()],
            },
        ];
        let dataset = SortTable {
            id,
            name,
            description,
            columns,
            rows,
        };

        assert_eq!(dataset.id, "123");
        assert_eq!(dataset.name, "Test");
        assert_eq!(dataset.description, "Test");
        assert_eq!(dataset.columns.len(), 2);
        assert_eq!(dataset.rows.len(), 2);
    }

    #[test]
    fn from_yaml() {
        let yaml_data = include_str!("../../../../assets/personal_pronouns.yml");
        let dataset: SortTable = serde_yaml::from_str(yaml_data).unwrap();
        assert_eq!(dataset.id, "personal_pronouns");
        assert_eq!(dataset.name, "Personal Pronouns");
        assert_eq!(
            dataset.description,
            "Match the personal pronouns in their correct case (Nominativ, Akkusativ, Dativ)"
        );
        assert_eq!(dataset.columns.len(), 3);
        assert_eq!(dataset.rows.len(), 6);
    }

    #[test]
    fn default_dataset() {
        let dataset = SortTable::default();
        assert_eq!(dataset.id, "personal_pronouns");
        assert_eq!(dataset.name, "Personal Pronouns");
        assert_eq!(
            dataset.description,
            "Match the personal pronouns in their correct case (Nominativ, Akkusativ, Dativ)"
        );
        assert_eq!(dataset.columns.len(), 3);
        assert_eq!(dataset.rows.len(), 6);
    }
}
