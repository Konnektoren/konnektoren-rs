use super::EmbeddedSource;

/// Embedded default content behind the `default-assets` feature.
///
/// With the feature enabled (default), this [`EmbeddedSource`] carries every
/// YAML that a `Default` impl in this crate falls back to — challenge
/// examples, the Konnektoren game path. With the feature disabled the source
/// is empty, [`default_asset`] returns `None`, and those `Default` impls
/// produce empty values instead; the binary then ships none of the content.
///
/// This is the per-kind embed-feature pattern: the feature *adds* an
/// embedded source, disabling it never removes the fetch-based ways of
/// loading the same files.
#[cfg(feature = "default-assets")]
pub static DEFAULT_ASSETS: EmbeddedSource = EmbeddedSource::new(&[
    (
        "konnektoren.yml",
        include_str!("../../../assets/konnektoren.yml").as_bytes(),
    ),
    (
        "konnektoren_path.yml",
        include_str!("../../../assets/konnektoren_path.yml").as_bytes(),
    ),
    (
        "articles-1.yml",
        include_str!("../../../assets/articles-1.yml").as_bytes(),
    ),
    (
        "past-tense.yml",
        include_str!("../../../assets/past-tense.yml").as_bytes(),
    ),
    (
        "sentence_structure.yml",
        include_str!("../../../assets/sentence_structure.yml").as_bytes(),
    ),
    (
        "dialog_begruessung.yml",
        include_str!("../../../assets/dialog_begruessung.yml").as_bytes(),
    ),
    (
        "personal_pronouns.yml",
        include_str!("../../../assets/personal_pronouns.yml").as_bytes(),
    ),
    (
        "personal_pronouns_info.yml",
        include_str!("../../../assets/personal_pronouns_info.yml").as_bytes(),
    ),
    (
        "gap_fill_default.yml",
        include_str!("../../../assets/gap_fill_default.yml").as_bytes(),
    ),
    (
        "contextual_choice_default.yml",
        include_str!("../../../assets/contextual_choice_default.yml").as_bytes(),
    ),
    (
        "ordering_default.yml",
        include_str!("../../../assets/ordering_default.yml").as_bytes(),
    ),
    (
        "placeholder_default.yml",
        include_str!("../../../assets/placeholder_default.yml").as_bytes(),
    ),
    (
        "custom_default.yml",
        include_str!("../../../assets/custom_default.yml").as_bytes(),
    ),
    (
        "vocabulary_default.yml",
        include_str!("../../../assets/vocabulary_default.yml").as_bytes(),
    ),
]);

/// Embedded default content behind the `default-assets` feature.
///
/// The feature is disabled — the source is empty and [`default_asset`]
/// always returns `None`.
#[cfg(not(feature = "default-assets"))]
pub static DEFAULT_ASSETS: EmbeddedSource = EmbeddedSource::new(&[]);

/// Parses the embedded default asset at `path` from [`DEFAULT_ASSETS`].
///
/// Returns `None` when the `default-assets` feature is disabled (or the
/// path is not embedded) — callers fall back to an empty value. Synchronous
/// on purpose: `Default` impls cannot await, and embedded bytes need no I/O.
///
/// # Panics
///
/// Panics if the embedded bytes fail to parse — the data ships inside the
/// binary, so that is a build-time data error, never a runtime condition.
pub fn default_asset<T: serde::de::DeserializeOwned>(path: &str) -> Option<T> {
    DEFAULT_ASSETS.get(path).map(|bytes| {
        serde_yaml::from_slice(bytes)
            .unwrap_or_else(|e| panic!("embedded default asset {path} is invalid YAML: {e}"))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "default-assets")]
    #[test]
    fn default_assets_are_embedded_and_parse() {
        use crate::challenges::ContextualChoice;

        let choice: ContextualChoice = default_asset("contextual_choice_default.yml").unwrap();
        assert!(!choice.items.is_empty());
    }

    #[cfg(feature = "default-assets")]
    #[test]
    fn default_assets_list_paths() {
        assert!(DEFAULT_ASSETS.paths().any(|p| p == "konnektoren.yml"));
        assert!(DEFAULT_ASSETS.paths().any(|p| p == "konnektoren_path.yml"));
    }

    #[cfg(not(feature = "default-assets"))]
    #[test]
    fn default_assets_are_empty_without_feature() {
        assert_eq!(DEFAULT_ASSETS.paths().count(), 0);
        assert!(default_asset::<serde_yaml::Value>("konnektoren.yml").is_none());
    }

    #[test]
    fn unknown_path_is_none() {
        assert!(default_asset::<serde_yaml::Value>("nope.yml").is_none());
    }
}
