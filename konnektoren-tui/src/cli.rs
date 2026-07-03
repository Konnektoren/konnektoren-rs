use std::path::PathBuf;

use clap::Parser;
use konnektoren_core::session::Session;

use crate::manifest_assets::{
    MANIFEST_ENV_VAR, ManifestSessionLoader, ManifestSource, Result as ManifestResult,
};

#[derive(Debug, Clone, Parser, PartialEq, Eq)]
#[command(
    name = "konnektoren-tui",
    version,
    about = "Run Konnektoren from the terminal"
)]
pub struct Cli {
    #[arg(
        long,
        env = MANIFEST_ENV_VAR,
        value_name = "FILE",
        help = "Path to a Konnektoren manifest YAML file"
    )]
    pub manifest: Option<PathBuf>,
}

impl Cli {
    pub fn manifest_source(&self) -> Option<ManifestSource> {
        self.manifest.as_ref().cloned().map(ManifestSource::new)
    }

    pub fn load_session(&self) -> ManifestResult<Option<Session>> {
        self.manifest_source()
            .map(|source| ManifestSessionLoader.load(&source))
            .transpose()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn parses_manifest_argument() -> std::result::Result<(), String> {
        let cli = Cli::try_parse_from(["konnektoren-tui", "--manifest", "game.yml"])
            .map_err(|err| err.to_string())?;

        assert_eq!(cli.manifest, Some(PathBuf::from("game.yml")));
        Ok(())
    }

    #[test]
    fn help_contains_manifest_option() {
        let help = Cli::command().render_long_help().to_string();

        assert!(help.contains("--manifest <FILE>"));
        assert!(help.contains("Path to a Konnektoren manifest YAML file"));
    }
}
