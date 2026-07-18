use thiserror::Error;

#[derive(Error, Debug)]
pub enum KonnektorenError {
    #[error("Challenge error: {0}")]
    Challenge(#[from] crate::challenges::ChallengeError),

    #[error("Game error: {0}")]
    Game(#[from] crate::game::GameError),

    #[error("Persistence error: {0}")]
    Persistence(#[from] crate::persistence::PersistenceError),

    #[error("Command error: {0}")]
    Command(#[from] crate::commands::CommandError),

    #[error("Asset error: {0}")]
    Asset(#[from] crate::assets::AssetError),

    #[cfg(feature = "certificates")]
    #[error("Certificate error: {0}")]
    Certificate(#[from] crate::certificates::CertificateError),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub type Result<T> = std::result::Result<T, KonnektorenError>;
