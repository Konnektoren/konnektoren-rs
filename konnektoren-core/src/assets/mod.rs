//! Minimal asset-loading abstraction: pluggable **async** sources, **sync**
//! formats.
//!
//! Mirrors the proven three-layer split (bevy's `AssetReader` / `AssetLoader`
//! / `AssetServer`) without any engine dependency:
//!
//! - [`AssetSource`] — *where bytes come from*. **Async by contract**: the
//!   trait is written for backends that must await (browser `fetch`,
//!   filesystem), and its futures are not required to be `Send` so wasm
//!   backends fit. The existing
//!   [`AssetLoader`](crate::asset_loader::AssetLoader) (`Url` fetch in CSR,
//!   filesystem in SSR) implements it.
//! - [`AssetFormat`] — *bytes → typed asset*. Sync, pure, trivially
//!   unit-testable; knows nothing about sources.
//! - [`EmbeddedSource`] — core's own implementation: compile-time embedded
//!   bytes via `include_bytes!` / `include_str!`. Answering from memory, it
//!   resolves instantly behind the same async interface, and needs no cache.
//!
//! Caching (`Arc<T>`, single-flight dedup), runtime fetch policies, zip
//! archives, per-kind embed features and an `AssetCollection` derive are
//! deliberately **not** here — they belong to a feature-rich implementation
//! crate built on these traits.
//!
//! Callers combine source and format through [`AssetSource::load`], whose
//! call shape stays identical regardless of the backend:
//!
//! ```
//! use konnektoren_core::assets::{AssetFormat, AssetSource, EmbeddedSource};
//!
//! /// Parses UTF-8 text files.
//! struct TextFormat;
//!
//! impl AssetFormat for TextFormat {
//!     type Asset = String;
//!     type Error = std::string::FromUtf8Error;
//!
//!     fn extensions(&self) -> &[&str] {
//!         &["txt", "md"]
//!     }
//!
//!     fn parse(&self, bytes: &[u8]) -> Result<Self::Asset, Self::Error> {
//!         String::from_utf8(bytes.to_vec())
//!     }
//! }
//!
//! // In real code the bytes come from include_bytes!("../assets/intro.md")
//! // or include_str!("../assets/intro.md").as_bytes().
//! static ASSETS: EmbeddedSource = EmbeddedSource::new(&[("intro.md", b"# Hallo")]);
//!
//! # async fn example() -> Result<(), konnektoren_core::assets::AssetError> {
//! let text: String = ASSETS.load(&TextFormat, "intro.md").await?;
//! assert_eq!(text, "# Hallo");
//! # Ok(())
//! # }
//! ```

mod error;
mod format;
mod source;

pub use error::AssetError;
pub use format::AssetFormat;
pub use source::{AssetSource, EmbeddedSource};
