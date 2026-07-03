mod app;
mod challenge_tabs;
mod challenge_widget;
#[cfg(feature = "cli")]
mod cli;
mod error;
mod manifest_assets;
mod map_widget;
mod options_widget;
mod results_widget;

#[cfg(feature = "crossterm")]
mod tui;

#[cfg(feature = "ssh")]
pub mod ssh_server;

pub mod prelude {
    pub use crate::app::App;

    #[cfg(feature = "cli")]
    pub use crate::cli::Cli;

    pub use crate::manifest_assets::{
        MANIFEST_ENV_VAR, ManifestSessionLoader, ManifestSource, ManifestSourceResolver,
        load_session_from_env,
    };

    #[cfg(feature = "crossterm")]
    pub use crate::tui::{Tui, init, restore};

    #[cfg(feature = "ssh")]
    pub use crate::ssh_server::SshServer;
}
