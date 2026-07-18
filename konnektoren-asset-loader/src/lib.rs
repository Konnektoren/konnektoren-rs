//! A framework-free asset server built on `konnektoren_core::assets`.
mod collection;
mod error;
mod server;
mod source;
pub use collection::AssetCollection;
pub use error::AssetLoadError;
#[cfg(feature = "derive")]
pub use konnektoren_asset_loader_derive::AssetCollection;
pub use konnektoren_core::assets::{AssetFormat, AssetSource};
pub use server::AssetServer;
#[cfg(feature = "zip")]
pub use source::ZipSource;
pub use source::{LayeredSource, SourceLayer};
