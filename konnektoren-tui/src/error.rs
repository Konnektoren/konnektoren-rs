use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Command execution error: {0}")]
    Command(#[from] konnektoren_core::commands::CommandError),

    #[error("UI error: {0}")]
    Ui(String),

    #[error("State error: {0}")]
    State(String),

    #[error("Manifest asset error: {0}")]
    ManifestAssets(#[from] crate::manifest_assets::ManifestAssetError),
}

pub type Result<T> = std::result::Result<T, Error>;
