//! Editorial content models for Konnektoren: FAQ and inbox/news messages.
//!
//! Deliberately independent of `konnektoren-core` — this content has no relationship to
//! challenges or game state. Depends only on `konnektoren-platform` for typed language
//! lookups ([`konnektoren_platform::i18n::Language`]).
//!
//! Loading is not yet wired up here — see the `Asset Loader` design note in the workspace
//! docs. For now, callers deserialize [`faq::FAQData`] / [`inbox::InboxData`] from YAML
//! however suits them (`include_str!` + `serde_yaml`, as the tests in this crate do).

pub mod faq;
pub mod inbox;
