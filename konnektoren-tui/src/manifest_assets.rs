use std::{
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
};

use konnektoren_core::{
    game::{Game, GamePath, GameState},
    player_profile::PlayerProfile,
    session::Session,
};
use konnektoren_platform::{
    cargo_package,
    manifest::{KonnektorenManifest, ManifestConfig},
};

pub const MANIFEST_ENV_VAR: &str = "KONNEKTOREN_MANIFEST";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestSource {
    path: PathBuf,
}

impl ManifestSource {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ManifestAssetError {
    #[error("--manifest requires a path")]
    MissingManifestArgument,

    #[error("failed to read manifest {path}: {source}")]
    ReadManifest {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("failed to parse manifest {path}: {message}")]
    ParseManifest { path: PathBuf, message: String },

    #[error("manifest {path} does not define any game_paths")]
    NoGamePaths { path: PathBuf },

    #[error("failed to read game path {path}: {source}")]
    ReadGamePath {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("failed to parse game path {path}: {source}")]
    ParseGamePath {
        path: PathBuf,
        source: serde_yaml::Error,
    },

    #[error("game path {path} does not contain any challenges")]
    EmptyGamePath { path: PathBuf },

    #[error("failed to create initial challenge {challenge_id}: {source}")]
    CreateInitialChallenge {
        challenge_id: String,
        source: konnektoren_core::game::GameError,
    },
}

pub type Result<T> = std::result::Result<T, ManifestAssetError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestSourceResolver {
    env_var: &'static str,
}

impl Default for ManifestSourceResolver {
    fn default() -> Self {
        Self {
            env_var: MANIFEST_ENV_VAR,
        }
    }
}

impl ManifestSourceResolver {
    pub fn resolve_from_env(&self) -> Result<Option<ManifestSource>> {
        self.resolve(std::env::args_os(), std::env::var_os(self.env_var))
    }

    pub fn resolve<I, S>(
        &self,
        args: I,
        env_value: Option<OsString>,
    ) -> Result<Option<ManifestSource>>
    where
        I: IntoIterator<Item = S>,
        S: Into<OsString>,
    {
        let args = args.into_iter().map(Into::into).collect::<Vec<_>>();
        if let Some(path) = cli_manifest_path(&args)? {
            return Ok(Some(ManifestSource::new(path)));
        }

        Ok(env_value
            .filter(|value| !value.is_empty())
            .map(ManifestSource::new))
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ManifestSessionLoader;

impl ManifestSessionLoader {
    pub fn load(&self, source: &ManifestSource) -> Result<Session> {
        let manifest = read_manifest(source.path())?;
        let manifest_dir = source.path().parent().unwrap_or_else(|| Path::new("."));
        let assets_dir = resolve_assets_dir(manifest_dir, manifest.asset_path());
        let game_paths = load_game_paths(source.path(), &assets_dir, manifest.game_paths())?;
        session_from_manifest(manifest, game_paths, source.path())
    }
}

pub fn load_session_from_env() -> Result<Option<Session>> {
    let resolver = ManifestSourceResolver::default();
    resolver
        .resolve_from_env()?
        .map(|source| ManifestSessionLoader.load(&source))
        .transpose()
}

fn cli_manifest_path(args: &[OsString]) -> Result<Option<PathBuf>> {
    let mut iter = args.iter().skip(1);
    while let Some(arg) = iter.next() {
        if arg == "--manifest" {
            return iter
                .next()
                .map(PathBuf::from)
                .map(Some)
                .ok_or(ManifestAssetError::MissingManifestArgument);
        }

        if let Some(value) = arg.to_string_lossy().strip_prefix("--manifest=") {
            return Ok(Some(PathBuf::from(value)));
        }
    }

    Ok(None)
}

fn read_manifest(path: &Path) -> Result<KonnektorenManifest> {
    let yaml = fs::read_to_string(path).map_err(|source| ManifestAssetError::ReadManifest {
        path: path.to_path_buf(),
        source,
    })?;

    KonnektorenManifest::figment(cargo_package!(), &yaml)
        .extract()
        .map_err(|source| ManifestAssetError::ParseManifest {
            path: path.to_path_buf(),
            message: source.to_string(),
        })
}

fn load_game_paths(
    manifest_path: &Path,
    assets_dir: &Path,
    game_path_names: &[String],
) -> Result<Vec<GamePath>> {
    if game_path_names.is_empty() {
        return Err(ManifestAssetError::NoGamePaths {
            path: manifest_path.to_path_buf(),
        });
    }

    game_path_names
        .iter()
        .map(|name| read_game_path(&resolve_relative_to(assets_dir, name)))
        .collect()
}

fn read_game_path(path: &Path) -> Result<GamePath> {
    let yaml = fs::read_to_string(path).map_err(|source| ManifestAssetError::ReadGamePath {
        path: path.to_path_buf(),
        source,
    })?;

    serde_yaml::from_str(&yaml).map_err(|source| ManifestAssetError::ParseGamePath {
        path: path.to_path_buf(),
        source,
    })
}

fn session_from_manifest(
    manifest: KonnektorenManifest,
    game_paths: Vec<GamePath>,
    manifest_path: &Path,
) -> Result<Session> {
    let first_challenge_id = game_paths
        .first()
        .and_then(|path| path.challenges.first())
        .map(|challenge| challenge.id.clone())
        .ok_or_else(|| ManifestAssetError::EmptyGamePath {
            path: manifest_path.to_path_buf(),
        })?;

    let game = Game {
        game_paths,
        ..Game::default()
    };
    let challenge = game
        .create_challenge(&first_challenge_id)
        .map_err(|source| ManifestAssetError::CreateInitialChallenge {
            challenge_id: first_challenge_id.clone(),
            source,
        })?;

    let player_id = if manifest.package().id.is_empty() {
        "konnektoren-tui".to_string()
    } else {
        manifest.package().id.clone()
    };
    let player_profile = PlayerProfile::new(player_id.clone());

    Ok(Session {
        id: player_id,
        player_profile,
        game_state: GameState {
            game,
            challenge,
            current_game_path: 0,
            current_challenge_index: 0,
            current_task_index: 0,
        },
    })
}

fn resolve_relative_to(base: &Path, value: &str) -> PathBuf {
    let path = PathBuf::from(value);
    if path.is_absolute() {
        path
    } else {
        base.join(path)
    }
}

fn resolve_assets_dir(manifest_dir: &Path, asset_path: &str) -> PathBuf {
    let path = PathBuf::from(asset_path);
    if path.is_absolute() {
        path
    } else if manifest_dir.ends_with(&path) {
        manifest_dir.to_path_buf()
    } else {
        manifest_dir.join(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn os(value: &str) -> OsString {
        OsString::from(value)
    }

    #[test]
    fn cli_manifest_wins_over_env_var() -> Result<()> {
        let resolver = ManifestSourceResolver::default();
        let source = resolver
            .resolve(
                [os("konnektoren-tui"), os("--manifest"), os("cli.yml")],
                Some(os("env.yml")),
            )?
            .ok_or(ManifestAssetError::MissingManifestArgument)?;

        assert_eq!(source.path(), Path::new("cli.yml"));
        Ok(())
    }

    #[test]
    fn env_var_used_when_cli_manifest_is_absent() -> Result<()> {
        let resolver = ManifestSourceResolver::default();
        let source = resolver
            .resolve([os("konnektoren-tui")], Some(os("env.yml")))?
            .ok_or(ManifestAssetError::MissingManifestArgument)?;

        assert_eq!(source.path(), Path::new("env.yml"));
        Ok(())
    }

    #[test]
    fn missing_manifest_argument_is_error() {
        let resolver = ManifestSourceResolver::default();
        let result = resolver.resolve([os("konnektoren-tui"), os("--manifest")], None);

        assert!(matches!(
            result,
            Err(ManifestAssetError::MissingManifestArgument)
        ));
    }

    #[test]
    fn manifest_loader_builds_session_from_relative_asset_path() -> Result<()> {
        let temp_dir = tempfile::tempdir().map_err(|source| ManifestAssetError::ReadManifest {
            path: PathBuf::from("tempdir"),
            source,
        })?;
        let assets_dir = temp_dir.path().join("content");
        fs::create_dir(&assets_dir).map_err(|source| ManifestAssetError::ReadManifest {
            path: assets_dir.clone(),
            source,
        })?;

        let game_path_yaml = r#"
id: custom-path
name: Custom Path
challenges:
  - id: custom-first
    name: Custom First
    description: First custom challenge
    challenge: konnektoren
    tasks: 1
    unlock_points: 0
"#;
        let game_path_path = assets_dir.join("level.yml");
        fs::File::create(&game_path_path)
            .and_then(|mut file| file.write_all(game_path_yaml.as_bytes()))
            .map_err(|source| ManifestAssetError::ReadGamePath {
                path: game_path_path.clone(),
                source,
            })?;

        let manifest_yaml = r#"
package:
  id: custom-package
  name: Custom Package
assets:
  path: content
game_paths:
  - level.yml
"#;
        let manifest_path = temp_dir.path().join("konnektoren.manifest.yml");
        fs::File::create(&manifest_path)
            .and_then(|mut file| file.write_all(manifest_yaml.as_bytes()))
            .map_err(|source| ManifestAssetError::ReadManifest {
                path: manifest_path.clone(),
                source,
            })?;

        let session = ManifestSessionLoader.load(&ManifestSource::new(&manifest_path))?;

        assert_eq!(session.id, "custom-package");
        assert_eq!(
            session.game_state.game.game_paths[0].challenge_ids(),
            vec!["custom-first".to_string()]
        );
        assert_eq!(
            session.game_state.challenge.challenge_config.id,
            "custom-first"
        );
        Ok(())
    }

    #[test]
    fn manifest_loader_does_not_duplicate_asset_dir_when_manifest_lives_inside_it() -> Result<()> {
        let temp_dir = tempfile::tempdir().map_err(|source| ManifestAssetError::ReadManifest {
            path: PathBuf::from("tempdir"),
            source,
        })?;
        let assets_dir = temp_dir.path().join("assets");
        fs::create_dir(&assets_dir).map_err(|source| ManifestAssetError::ReadManifest {
            path: assets_dir.clone(),
            source,
        })?;

        let game_path_yaml = r#"
id: assets-path
name: Assets Path
challenges:
  - id: assets-first
    name: Assets First
    description: First assets challenge
    challenge: konnektoren
    tasks: 1
    unlock_points: 0
"#;
        let game_path_path = assets_dir.join("level.yml");
        fs::File::create(&game_path_path)
            .and_then(|mut file| file.write_all(game_path_yaml.as_bytes()))
            .map_err(|source| ManifestAssetError::ReadGamePath {
                path: game_path_path.clone(),
                source,
            })?;

        let manifest_yaml = r#"
package:
  id: assets-package
  name: Assets Package
assets:
  path: assets
game_paths:
  - level.yml
"#;
        let manifest_path = assets_dir.join("konnektoren.manifest.yml");
        fs::File::create(&manifest_path)
            .and_then(|mut file| file.write_all(manifest_yaml.as_bytes()))
            .map_err(|source| ManifestAssetError::ReadManifest {
                path: manifest_path.clone(),
                source,
            })?;

        let session = ManifestSessionLoader.load(&ManifestSource::new(&manifest_path))?;

        assert_eq!(session.id, "assets-package");
        assert_eq!(
            session.game_state.game.game_paths[0].challenge_ids(),
            vec!["assets-first".to_string()]
        );
        Ok(())
    }
}
