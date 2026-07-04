//! Umbrella crate for the Konnektoren workspace.
//!
//! Re-exports the individual `konnektoren-*` crates behind feature flags so
//! downstream consumers (yew, bevy, TUI, MCP server, …) can depend on a
//! single crate and a single version instead of tracking each workspace
//! member independently.
//!
//! # Features
//!
//! - `core` (default) — [`core`], the language-learning engine: challenges,
//!   game state, sessions, commands, controller, persistence.
//! - `platform` (default) — [`domain`], [`i18n`], [`manifest`].
//! - `achievements` — [`achievements`], gated behind `konnektoren-core`'s
//!   `achievements` feature.
//! - `certificates` — [`certificates`], gated behind `konnektoren-core`'s
//!   `certificates` feature.
//! - `marketplace` — [`marketplace`], gated behind `konnektoren-core`'s
//!   `marketplace` feature.
//!
//! Module paths are stable across feature combinations even as the
//! underlying crate layout changes — e.g. `achievements` re-exports from
//! `konnektoren-core` today, but could move to its own crate later without
//! consumers changing an import.

#[cfg(feature = "core")]
pub use konnektoren_core as core;

#[cfg(feature = "platform")]
pub use konnektoren_platform as platform;

#[cfg(feature = "achievements")]
pub use konnektoren_core::achievements;

#[cfg(feature = "certificates")]
pub use konnektoren_core::certificates;

#[cfg(feature = "marketplace")]
pub use konnektoren_core::marketplace;

#[cfg(feature = "platform")]
pub use konnektoren_platform::{domain, i18n, manifest};

/// Merged prelude of [`konnektoren_core::prelude`] and
/// [`konnektoren_platform::prelude`].
///
/// `certificates` and `marketplace` are deliberately left out: certificates'
/// `error::Result` would collide with core's `Result` under a glob import.
pub mod prelude {
    #[cfg(feature = "core")]
    pub use konnektoren_core::prelude::*;

    #[cfg(feature = "platform")]
    pub use konnektoren_platform::prelude::*;
}
