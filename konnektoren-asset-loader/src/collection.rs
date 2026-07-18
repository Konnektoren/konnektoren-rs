use crate::{AssetLoadError, AssetServer};
use konnektoren_core::assets::{AssetFormat, AssetSource};
use std::sync::Arc;
pub trait AssetCollection {
    const FOLDER: &'static str;
    fn asset_path(name: &str) -> String {
        format!(
            "{}/{}",
            Self::FOLDER.trim_end_matches('/'),
            name.trim_start_matches('/')
        )
    }
    fn load<'a, S, F>(
        server: &'a AssetServer<S>,
        format: F,
        name: &'a str,
    ) -> impl std::future::Future<Output = Result<Arc<F::Asset>, AssetLoadError>> + 'a
    where
        S: AssetSource + ?Sized + 'static,
        F: AssetFormat + Clone + 'static,
        F::Asset: Send + Sync + 'static,
    {
        async move { server.load(format, Self::asset_path(name)).await }
    }
}
