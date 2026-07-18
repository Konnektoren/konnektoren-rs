pub mod analytics;
pub mod assets;
pub mod challenges;
pub mod commands;
pub mod controller;
pub mod error;
pub mod events;
pub mod game;
pub mod persistence;
pub mod player_profile;
pub mod session;

#[cfg(feature = "achievements")]
pub mod achievements;

#[cfg(feature = "certificates")]
pub mod certificates;

#[cfg(feature = "js")]
pub mod konnektoren_js;

#[cfg(feature = "marketplace")]
pub mod marketplace;

/// experience points
pub type Xp = u32;

/// This is a prelude module that re-exports the most important types and traits.
pub mod prelude {
    pub use crate::Xp;
    pub use crate::analytics::metrics::Metric;
    #[cfg(feature = "csr")]
    pub use crate::assets::UrlSource;
    pub use crate::assets::{
        AssetError, AssetFormat, AssetSource, DefaultAssetSource, EmbeddedSource, FileSource,
    };
    pub use crate::challenges::Challenge;
    pub use crate::challenges::ChallengeConfig;
    pub use crate::challenges::ChallengeFactory;
    pub use crate::challenges::ChallengeInput;
    pub use crate::challenges::ChallengeResult;
    pub use crate::challenges::ChallengeType;
    pub use crate::challenges::ChallengeVariant;
    pub use crate::challenges::Dialog;
    pub use crate::challenges::DialogAnswer;
    pub use crate::challenges::DialogTurn;
    pub use crate::challenges::MultipleChoice;
    pub use crate::challenges::MultipleChoiceOption;
    pub use crate::challenges::Performance;
    pub use crate::challenges::Question;
    pub use crate::challenges::Speaker;
    pub use crate::commands::error::CommandError;
    pub use crate::controller::GameController;
    pub use crate::controller::error::ControllerError;
    pub use crate::error::{KonnektorenError, Result};
    pub use crate::game::Game;
    pub use crate::game::GamePath;
    pub use crate::persistence::error::PersistenceError;
    pub use crate::player_profile::PlayerProfile;
    pub use crate::session::Session;

    #[cfg(feature = "achievements")]
    pub use crate::achievements::*;

    #[cfg(feature = "js")]
    pub use crate::konnektoren_js::*;

    #[cfg(feature = "marketplace")]
    pub use crate::marketplace::*;
}

#[cfg(feature = "js")]
#[cfg(target_arch = "wasm32")]
#[cfg(test)]
mod tests {
    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);
}
