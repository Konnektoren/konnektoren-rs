use super::{AssetError, AssetFormat};
use crate::asset_loader::AssetLoader;

/// Delivers raw bytes for a path — the only async piece of the pipeline.
///
/// Implementations decide *where* bytes come from: HTTP fetch or filesystem
/// (both via [`AssetLoader`]), compile-time embeds ([`EmbeddedSource`]), or —
/// in richer implementation crates — zip archives and source chains. Callers
/// never branch on the backend; they pick which source value to construct
/// (or get one injected) and the call shape stays the same.
///
/// The trait is async by contract because real backends must await (browser
/// `fetch`, filesystem reads); instant backends like [`EmbeddedSource`]
/// simply resolve immediately. The returned futures are not required to be
/// `Send` so that wasm-only backends can implement this trait.
pub trait AssetSource {
    /// Loads the raw bytes at `path`, relative to the source's root.
    #[allow(async_fn_in_trait)]
    async fn load_bytes(&self, path: &str) -> Result<Vec<u8>, AssetError>;

    /// Loads the bytes at `path` and parses them with `format`.
    ///
    /// Provided method — implementors only supply
    /// [`load_bytes`](AssetSource::load_bytes).
    #[allow(async_fn_in_trait)]
    async fn load<F: AssetFormat>(
        &self,
        format: &F,
        path: &str,
    ) -> Result<F::Asset, AssetError> {
        let bytes = self.load_bytes(path).await?;
        format
            .parse(&bytes)
            .map_err(|source| AssetError::parse(path, source))
    }
}

/// The existing runtime loader is an [`AssetSource`]: `Url` fetch in CSR,
/// filesystem search in SSR — chosen by construction, invisible to callers
/// of the trait.
impl AssetSource for AssetLoader {
    async fn load_bytes(&self, path: &str) -> Result<Vec<u8>, AssetError> {
        self.load_binary(path)
            .await
            .map_err(|err| AssetError::Load {
                path: path.into(),
                message: err.to_string(),
            })
    }
}

/// Compile-time embedded [`AssetSource`] — core's own minimal backend.
///
/// Entries are `(path, bytes)` pairs, typically produced with
/// `include_bytes!` (or `include_str!(..).as_bytes()`), so the asset ships
/// inside the binary and loading never fails at runtime for known paths.
///
/// # Example
///
/// ```
/// use konnektoren_core::assets::{AssetSource, EmbeddedSource};
///
/// static ASSETS: EmbeddedSource = EmbeddedSource::new(&[
///     // in real code: ("konnektoren.yml", include_bytes!("../assets/konnektoren.yml")),
///     ("konnektoren.yml", b"id: konnektoren"),
/// ]);
///
/// # async fn example() -> Result<(), konnektoren_core::assets::AssetError> {
/// let bytes = ASSETS.load_bytes("konnektoren.yml").await?;
/// assert_eq!(bytes, b"id: konnektoren");
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Copy)]
pub struct EmbeddedSource {
    entries: &'static [(&'static str, &'static [u8])],
}

impl EmbeddedSource {
    /// Creates a source over static `(path, bytes)` entries.
    pub const fn new(entries: &'static [(&'static str, &'static [u8])]) -> Self {
        Self { entries }
    }

    /// Returns the embedded bytes for `path`, if present.
    pub fn get(&self, path: &str) -> Option<&'static [u8]> {
        self.entries
            .iter()
            .find(|(entry_path, _)| *entry_path == path)
            .map(|(_, bytes)| *bytes)
    }

    /// Iterates over all embedded paths.
    pub fn paths(&self) -> impl Iterator<Item = &'static str> {
        self.entries.iter().map(|(path, _)| *path)
    }
}

impl AssetSource for EmbeddedSource {
    async fn load_bytes(&self, path: &str) -> Result<Vec<u8>, AssetError> {
        self.get(path)
            .map(<[u8]>::to_vec)
            .ok_or_else(|| AssetError::NotFound { path: path.into() })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Utf8Format;

    impl AssetFormat for Utf8Format {
        type Asset = String;
        type Error = std::string::FromUtf8Error;

        fn extensions(&self) -> &[&str] {
            &["txt"]
        }

        fn parse(&self, bytes: &[u8]) -> Result<Self::Asset, Self::Error> {
            String::from_utf8(bytes.to_vec())
        }
    }

    static ASSETS: EmbeddedSource = EmbeddedSource::new(&[
        ("greeting.txt", b"hallo welt"),
        ("broken.txt", &[0xff, 0xfe]),
    ]);

    #[tokio::test]
    async fn embedded_loads_known_path() {
        let bytes = ASSETS.load_bytes("greeting.txt").await.unwrap();
        assert_eq!(bytes, b"hallo welt");
    }

    #[tokio::test]
    async fn embedded_missing_path_is_not_found() {
        let err = ASSETS.load_bytes("missing.yml").await.unwrap_err();
        assert!(matches!(err, AssetError::NotFound { path } if path == "missing.yml"));
    }

    #[test]
    fn embedded_lists_paths() {
        let paths: Vec<_> = ASSETS.paths().collect();
        assert_eq!(paths, vec!["greeting.txt", "broken.txt"]);
    }

    #[tokio::test]
    async fn load_parses_bytes_through_format() {
        let text = ASSETS.load(&Utf8Format, "greeting.txt").await.unwrap();
        assert_eq!(text, "hallo welt");
    }

    #[tokio::test]
    async fn load_wraps_parse_errors() {
        let err = ASSETS.load(&Utf8Format, "broken.txt").await.unwrap_err();
        assert!(matches!(err, AssetError::Parse { path, .. } if path == "broken.txt"));
    }

    #[tokio::test]
    async fn asset_loader_file_variant_is_a_source() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("hello.txt"), "from disk").unwrap();

        let loader = AssetLoader::from_dir(dir.path());
        let bytes = loader.load_bytes("hello.txt").await.unwrap();
        assert_eq!(bytes, b"from disk");

        let text = loader.load(&Utf8Format, "hello.txt").await.unwrap();
        assert_eq!(text, "from disk");
    }

    #[tokio::test]
    async fn asset_loader_missing_file_maps_to_load_error() {
        let dir = tempfile::tempdir().unwrap();
        let loader = AssetLoader::from_dir(dir.path());

        let err = loader.load_bytes("missing.yml").await.unwrap_err();
        assert!(matches!(err, AssetError::Load { path, .. } if path == "missing.yml"));
    }
}
