pub mod error;
pub mod i18n_config;
pub mod language;
pub mod translation;
pub mod translation_asset;
pub mod translation_map;

pub use error::AssetLoadError;
pub use i18n_config::I18nConfig;
pub use language::Language;
pub use translation::Translation;
pub use translation_asset::{
    CombinedTranslationAsset, I18nAssets, JsonTranslationAsset, TranslationAsset,
    YamlTranslationAsset,
};
pub use translation_map::TranslationMap;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_asset_loading() {
        let asset = JsonTranslationAsset::<I18nAssets>::new();
        let translations = asset.load_translations();
        assert!(translations.contains_key("en"));
        assert!(translations.contains_key("de"));
        assert_eq!(translations["en"]["Language"], "Language");
    }

    #[test]
    fn test_yaml_asset_loading() {
        let asset = YamlTranslationAsset::<I18nAssets>::new("i18n.yml");
        let translations = asset.load_translations();
        assert!(translations.contains_key("en"));
        assert!(translations.contains_key("de"));
        assert_eq!(translations["en"]["Description"], "Description");
    }

    #[test]
    fn test_config_with_assets() {
        let config = I18nConfig::with_assets(JsonTranslationAsset::<I18nAssets>::new());
        assert_eq!(config.get_translation("Language", None), "Language");
    }
}
