//! Editorial content models for Konnektoren: FAQ and inbox/news messages.
//!
//! Depends on `konnektoren-platform` for i18n primitives:
//! [`Language`](konnektoren_platform::i18n::Language) for typed language lookups and
//! [`TranslationMap`](konnektoren_platform::i18n::TranslationMap) for the text-keyed
//! `i18n:` blocks carried by each content item (same format as the challenge
//! translation YAML files).
//!
//! Enable the `schema` feature to derive `schemars::JsonSchema` on all models
//! (for generating server API schemas).
//!
//! Loading is not yet wired up here — see the `Asset Loader` design note in the workspace
//! docs. For now, callers deserialize [`faq::FaqData`] / [`inbox::InboxData`] from YAML
//! however suits them (`include_str!` + `serde_yaml`, as the tests in this crate do).

pub mod faq;
pub mod inbox;
pub mod tag;

pub use faq::{Faq, FaqData};
pub use inbox::{InboxData, Message};
pub use tag::Tag;
