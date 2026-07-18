use super::{AssetError, AssetFormat};
use std::path::PathBuf;

#[cfg(feature = "csr")]
use gloo::net::http::Request;

/// Delivers raw bytes for a path — the only async piece of the pipeline.
///
/// Implementations decide *where* bytes come from: compile-time embeds
/// ([`EmbeddedSource`]), the filesystem ([`FileSource`]), HTTP fetch
/// ([`UrlSource`], CSR), or — in richer implementation crates — zip archives
/// and source chains. Callers never branch on the backend; they pick which
/// source value to construct (or get one injected) and the call shape stays
/// the same.
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
    async fn load<F: AssetFormat>(&self, format: &F, path: &str) -> Result<F::Asset, AssetError> {
        let bytes = self.load_bytes(path).await?;
        format
            .parse(&bytes)
            .map_err(|source| AssetError::parse(path, source))
    }
}

/// The source used when none is injected: [`UrlSource`] under the `csr`
/// feature (browser fetch from `/assets/`), [`FileSource`] everywhere else
/// (SSR, native, tests).
#[cfg(feature = "csr")]
pub type DefaultAssetSource = UrlSource;

/// The source used when none is injected: [`UrlSource`] under the `csr`
/// feature (browser fetch from `/assets/`), [`FileSource`] everywhere else
/// (SSR, native, tests).
#[cfg(not(feature = "csr"))]
pub type DefaultAssetSource = FileSource;

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

/// Filesystem [`AssetSource`] for SSR, native binaries and tests.
///
/// Tries each base directory in order and reads the first hit; an absolute
/// `path` that exists is read directly as a last resort.
///
/// [`Default`] searches `$BUILD_DIR` (when set), the current directory and
/// `assets/`.
#[derive(Debug, Clone)]
pub struct FileSource {
    base_dirs: Vec<PathBuf>,
}

impl FileSource {
    /// Creates a source that searches `base_dirs` in order.
    pub fn new(base_dirs: Vec<PathBuf>) -> Self {
        Self { base_dirs }
    }

    /// Creates a source with a single base directory.
    pub fn from_dir(dir: impl Into<PathBuf>) -> Self {
        Self {
            base_dirs: vec![dir.into()],
        }
    }

    /// The directories searched, in order.
    pub fn base_dirs(&self) -> &[PathBuf] {
        &self.base_dirs
    }
}

impl Default for FileSource {
    fn default() -> Self {
        let mut base_dirs = Vec::new();
        if let Ok(build_dir) = std::env::var("BUILD_DIR") {
            base_dirs.push(PathBuf::from(build_dir));
        }
        base_dirs.push(PathBuf::from("./"));
        base_dirs.push(PathBuf::from("assets"));
        Self { base_dirs }
    }
}

impl AssetSource for FileSource {
    async fn load_bytes(&self, path: &str) -> Result<Vec<u8>, AssetError> {
        for base_dir in &self.base_dirs {
            let file_path = base_dir.join(path);
            if file_path.exists() {
                return std::fs::read(&file_path).map_err(|source| AssetError::Load {
                    path: file_path.display().to_string(),
                    message: source.to_string(),
                });
            }
        }

        let path_buf = PathBuf::from(path);
        if path_buf.is_absolute() && path_buf.exists() {
            return std::fs::read(&path_buf).map_err(|source| AssetError::Load {
                path: path_buf.display().to_string(),
                message: source.to_string(),
            });
        }

        Err(AssetError::NotFound { path: path.into() })
    }
}

/// HTTP-fetch [`AssetSource`] for CSR (browser) builds.
///
/// Joins `base_url` and the requested path with exactly one `/` between
/// them, so both `"/assets/"` and `"/assets"` work as a base.
///
/// [`Default`] fetches from `/assets/`.
#[cfg(feature = "csr")]
#[derive(Debug, Clone)]
pub struct UrlSource {
    base_url: String,
}

#[cfg(feature = "csr")]
impl UrlSource {
    /// Creates a source fetching relative to `base_url`.
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
        }
    }

    /// The base URL requests are made against.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    fn url_for(&self, path: &str) -> String {
        let normalized_path = path.trim_start_matches('/');
        if self.base_url.ends_with('/') {
            format!("{}{}", self.base_url, normalized_path)
        } else {
            format!("{}/{}", self.base_url, normalized_path)
        }
    }
}

#[cfg(feature = "csr")]
impl Default for UrlSource {
    fn default() -> Self {
        Self {
            base_url: "/assets/".to_string(),
        }
    }
}

#[cfg(feature = "csr")]
impl AssetSource for UrlSource {
    async fn load_bytes(&self, path: &str) -> Result<Vec<u8>, AssetError> {
        let url = self.url_for(path);

        let response = Request::get(&url)
            .send()
            .await
            .map_err(|e| AssetError::Load {
                path: url.clone(),
                message: format!("failed to send request: {}", e),
            })?;

        if response.status() != 200 {
            return Err(AssetError::Load {
                path: url,
                message: format!("status {}", response.status()),
            });
        }

        response.binary().await.map_err(|e| AssetError::Load {
            path: url,
            message: format!("failed to read response: {}", e),
        })
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
    async fn file_source_loads_from_base_dir() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("hello.txt"), "from disk").unwrap();

        let source = FileSource::from_dir(dir.path());
        let bytes = source.load_bytes("hello.txt").await.unwrap();
        assert_eq!(bytes, b"from disk");

        let text = source.load(&Utf8Format, "hello.txt").await.unwrap();
        assert_eq!(text, "from disk");
    }

    #[tokio::test]
    async fn file_source_searches_base_dirs_in_order() {
        let first = tempfile::tempdir().unwrap();
        let second = tempfile::tempdir().unwrap();
        std::fs::write(second.path().join("only_second.txt"), "from second").unwrap();
        std::fs::write(first.path().join("in_both.txt"), "from first").unwrap();
        std::fs::write(second.path().join("in_both.txt"), "from second").unwrap();

        let source = FileSource::new(vec![first.path().into(), second.path().into()]);
        assert_eq!(
            source.load_bytes("only_second.txt").await.unwrap(),
            b"from second"
        );
        assert_eq!(
            source.load_bytes("in_both.txt").await.unwrap(),
            b"from first"
        );
    }

    #[tokio::test]
    async fn file_source_loads_absolute_path_as_fallback() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("abs.txt"), "absolute").unwrap();

        let source = FileSource::from_dir("nonexistent-dir");
        let abs = dir.path().join("abs.txt");
        let bytes = source.load_bytes(abs.to_str().unwrap()).await.unwrap();
        assert_eq!(bytes, b"absolute");
    }

    #[tokio::test]
    async fn file_source_missing_file_is_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let source = FileSource::from_dir(dir.path());

        let err = source.load_bytes("missing.yml").await.unwrap_err();
        assert!(matches!(err, AssetError::NotFound { path } if path == "missing.yml"));
    }

    #[cfg(all(feature = "ssr", not(feature = "csr")))]
    #[tokio::test]
    async fn default_file_source_honors_build_dir() {
        let temp_dir = tempfile::tempdir().unwrap();
        let assets_dir = temp_dir.path().join("assets");
        std::fs::create_dir_all(&assets_dir).unwrap();
        std::fs::write(
            assets_dir.join("konnektoren.yml"),
            include_str!("../../../assets/konnektoren.yml"),
        )
        .unwrap();

        unsafe { std::env::set_var("BUILD_DIR", temp_dir.path().to_str().unwrap()) };
        let source = FileSource::default();
        unsafe { std::env::remove_var("BUILD_DIR") };

        let content = source.load_bytes("assets/konnektoren.yml").await.unwrap();
        let content_str = String::from_utf8(content).unwrap();
        assert!(content_str.contains("id: \"konnektoren\""));
        assert!(content_str.contains("name: \"Konnektoren\""));
    }

    #[cfg(feature = "csr")]
    mod csr {
        use super::*;
        use wasm_bindgen_test::*;

        wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

        #[test]
        fn url_source_normalizes_paths() {
            assert_eq!(
                UrlSource::new("/assets/").url_for("/konnektoren.yml"),
                "/assets/konnektoren.yml"
            );
            assert_eq!(
                UrlSource::new("https://example.com/static").url_for("konnektoren.yml"),
                "https://example.com/static/konnektoren.yml"
            );
        }

        #[wasm_bindgen_test]
        async fn url_source_default_fetches_from_assets() {
            // No server in the test environment — assert the failed fetch
            // reports the URL it tried, proving URL construction.
            let source = UrlSource::default();
            if let Err(e) = source.load_bytes("assets/konnektoren.yml").await {
                assert!(e.to_string().contains("assets/konnektoren.yml"));
            }
        }

        #[wasm_bindgen_test]
        async fn url_source_uses_custom_base() {
            let source = UrlSource::new("https://example.com/static");
            if let Err(e) = source.load_bytes("assets/konnektoren.yml").await {
                assert!(
                    e.to_string()
                        .contains("https://example.com/static/assets/konnektoren.yml")
                );
            }
        }
    }
}
