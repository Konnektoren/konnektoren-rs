pub mod domain;
pub mod extensions;
pub mod i18n;
pub mod konnektoren;

#[cfg(feature = "manifest")]
use figment2::Figment;
#[cfg(feature = "manifest")]
use figment2::providers::{Format, Serialized, Yaml};
use konnektoren_core::game::GamePath;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::fmt::Debug;

pub use domain::DomainManifest;
pub use extensions::{ManifestExtensions, ManifestSection, NoExtensions};
pub use i18n::I18nManifest;
pub use konnektoren::{KonnektorenManifest, KonnektorenSections};

/// The default manifest shipped with this crate (`assets/konnektoren.manifest.yml`).
pub const DEFAULT_MANIFEST: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/konnektoren.manifest.yml"
));

/// Identity of a deployment — mirrors Cargo's `[package]`.
#[derive(Debug, Clone, PartialEq, Default, Deserialize, Serialize)]
pub struct Package {
    pub id: String,
    #[serde(default)]
    pub version: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub authors: Vec<String>,
    #[serde(default)]
    pub repository: Option<String>,
    #[serde(default)]
    pub license: Option<String>,
    #[serde(default)]
    pub keywords: Vec<String>,
}

/// Where challenge/asset files live.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Assets {
    #[serde(default = "default_asset_path")]
    pub path: String,
}

impl Default for Assets {
    fn default() -> Self {
        Self {
            path: default_asset_path(),
        }
    }
}

fn default_asset_path() -> String {
    "assets".to_string()
}

/// Base deployment manifest.
///
/// `Ext` is a flat bundle of named sections — each field in `Ext` appears as a
/// top-level YAML key. Build `Ext` with the [`manifest_extensions!`] macro.
///
/// ```yaml
/// # konnektoren.manifest.yml
/// package:
///   id: my-game
///   name: My Game
///
/// assets:
///   path: challenges
///
/// game_paths:
///   - level_a1.yml
///
/// # sections from Ext — top-level, not nested:
/// domain:
///   hostname: my-game.example.com
///
/// i18n:
///   default_language: de
///   languages: [de, en]
/// ```
///
/// ## Defining a custom extension
///
/// ```rust,ignore
/// use konnektoren_platform::{manifest_extensions, manifest_section};
///
/// #[derive(Debug, Clone, PartialEq, Default, serde::Serialize, serde::Deserialize)]
/// pub struct ThemeConfig { pub dark_mode: bool }
/// manifest_section!(ThemeConfig, "theme");
///
/// manifest_extensions! {
///     pub struct MyExtensions {
///         pub theme: ThemeConfig,
///     }
/// }
///
/// type MyManifest = Manifest<MyExtensions>;
/// ```
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(bound(
    deserialize = "Ext: Default + DeserializeOwned",
    serialize = "Ext: Serialize"
))]
pub struct Manifest<Ext = KonnektorenSections>
where
    Ext: ManifestExtensions,
{
    #[serde(default)]
    pub package: Package,

    #[serde(default)]
    pub assets: Assets,

    /// Ordered list of game-path filenames, resolved relative to `assets.path`.
    #[serde(default)]
    pub game_paths: Vec<String>,

    /// All extension sections, flattened to top-level YAML keys.
    #[serde(flatten)]
    pub ext: Ext,
}

impl<Ext: ManifestExtensions> Default for Manifest<Ext> {
    fn default() -> Self {
        Self {
            package: Package::default(),
            assets: Assets::default(),
            game_paths: vec![],
            ext: Ext::default(),
        }
    }
}

/// Core accessors available on every manifest regardless of its extension type.
pub trait ManifestConfig {
    fn package(&self) -> &Package;
    fn asset_path(&self) -> &str;
    fn game_paths(&self) -> &[String];

    fn challenges_path(&self) -> String {
        format!("{}/challenges", self.asset_path())
    }

    fn load_game_paths<F, E>(&self, mut loader: F) -> Result<Vec<GamePath>, E>
    where
        F: FnMut(&str) -> Result<GamePath, E>,
    {
        self.game_paths().iter().map(|p| loader(p)).collect()
    }
}

impl<Ext: ManifestExtensions> ManifestConfig for Manifest<Ext> {
    fn package(&self) -> &Package {
        &self.package
    }

    fn asset_path(&self) -> &str {
        &self.assets.path
    }

    fn game_paths(&self) -> &[String] {
        &self.game_paths
    }
}

#[cfg(feature = "manifest")]
impl<Ext> Manifest<Ext>
where
    Ext: ManifestExtensions + Serialize + Debug + Clone + PartialEq,
{
    /// Build a [`Figment`] with all defaults seeded from `default_package` and
    /// `Ext::default()`, then overlaid with the given YAML string.
    ///
    /// Use [`cargo_package!`] to supply `default_package` so the env vars
    /// resolve in the *calling* crate.
    ///
    /// ```rust,ignore
    /// let manifest: KonnektorenManifest = Manifest::figment(
    ///     cargo_package!(),
    ///     include_str!("../assets/konnektoren.manifest.yml"),
    /// )
    /// .extract()?;
    /// ```
    pub fn figment(default_package: Package, yaml: &str) -> Figment {
        let defaults = Self {
            package: default_package,
            ..Default::default()
        };
        Figment::new()
            .merge(Serialized::defaults(defaults))
            .merge(Yaml::string(yaml))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MINIMAL_YAML: &str = r#"
package:
  id: test-deployment
  version: "1.0.0"
  name: Test

game_paths:
  - level_a1.yml
  - level_a2.yml
"#;

    #[test]
    fn test_minimal_manifest_deserializes() {
        let manifest: Manifest<NoExtensions> = serde_yaml::from_str(MINIMAL_YAML).unwrap();
        assert_eq!(manifest.package.id, "test-deployment");
        assert_eq!(manifest.game_paths, vec!["level_a1.yml", "level_a2.yml"]);
        assert_eq!(manifest.assets.path, "assets");
    }

    #[test]
    fn test_manifest_config_trait() {
        let manifest: Manifest<NoExtensions> = serde_yaml::from_str(MINIMAL_YAML).unwrap();
        assert_eq!(manifest.package().id, "test-deployment");
        assert_eq!(manifest.asset_path(), "assets");
        assert_eq!(manifest.game_paths().len(), 2);
    }

    #[test]
    fn test_custom_extensions_with_macro() {
        crate::manifest_extensions! {
            pub struct TestExt {
                pub theme: String,
            }
        }

        let yaml = r#"
package:
  id: ext-game
  name: Ext Game
assets:
  path: assets/challenges
theme: dark
game_paths:
  - level_a1.yml
"#;
        let manifest: Manifest<TestExt> = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(manifest.package.id, "ext-game");
        assert_eq!(manifest.assets.path, "assets/challenges");
        assert_eq!(manifest.ext.theme, "dark");
    }

    #[test]
    fn test_default_manifest_parses() {
        let manifest: KonnektorenManifest =
            serde_yaml::from_str(DEFAULT_MANIFEST).expect("DEFAULT_MANIFEST must be valid YAML");
        assert!(!manifest.game_paths.is_empty());
    }

    #[cfg(feature = "manifest")]
    #[test]
    fn test_figment_fills_package_from_defaults() {
        let default_package = Package {
            id: "my-game".to_string(),
            version: "2.0.0".to_string(),
            name: "My Game".to_string(),
            description: Some("A test game".to_string()),
            ..Default::default()
        };

        let yaml = r#"
assets:
  path: custom/challenges
game_paths:
  - level_a1.yml
"#;

        let manifest: Manifest<NoExtensions> =
            Manifest::<NoExtensions>::figment(default_package, yaml)
                .extract()
                .expect("figment extraction must succeed");

        assert_eq!(manifest.package.id, "my-game");
        assert_eq!(manifest.package.version, "2.0.0");
        assert_eq!(manifest.assets.path, "custom/challenges");
        assert_eq!(manifest.game_paths, vec!["level_a1.yml"]);
    }

    #[cfg(feature = "manifest")]
    #[test]
    fn test_figment_sections_seeded_from_defaults() {
        use crate::manifest::konnektoren::KonnektorenSections;

        let default_package = Package {
            id: "my-game".to_string(),
            ..Default::default()
        };

        // YAML only overrides hostname; i18n defaults should still be present
        let yaml = r#"
domain:
  code: test
  name: Test
  hostname: test.example.com
"#;

        let manifest: Manifest<KonnektorenSections> =
            Manifest::<KonnektorenSections>::figment(default_package, yaml)
                .extract()
                .expect("figment extraction must succeed");

        assert_eq!(
            manifest.ext.domain.as_ref().unwrap().hostname,
            "test.example.com"
        );
        // i18n defaults were seeded even though yaml didn't set them
        assert_eq!(manifest.ext.i18n.default_language, "en");
        assert_eq!(manifest.ext.i18n.path, "i18n");
    }
}
