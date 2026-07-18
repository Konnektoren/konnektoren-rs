/// Turns raw bytes into a typed asset.
///
/// The contract that keeps frontend adapters thin: **sync, bytes in, value
/// out** — no source knowledge, no async, no load context. A format written
/// once serves every source (file, fetch, embedded) and every frontend (a
/// bevy adapter is ~ten lines: read bytes, call [`parse`](AssetFormat::parse),
/// wrap in the engine's asset newtype).
///
/// # Relation to bevy's `AssetLoader`
///
/// Mirrors [`bevy::asset::AssetLoader`](https://docs.rs/bevy/latest/bevy/asset/trait.AssetLoader.html)
/// where that helps a future adapter, and deliberately diverges where bevy
/// solves engine problems this crate does not have:
///
/// - `extensions()` matches bevy: advisory, defaults to an empty list, and
///   loading a file with a non-matching extension is not an error.
/// - `type Error` plays the role of bevy's `Error: Into<BevyError>` — any
///   `std::error::Error` value, type-erased into
///   [`AssetError::Parse`](crate::assets::AssetError::Parse) by the caller.
/// - bevy's `type Settings` (per-load configuration, driven by `.meta`
///   files) has no counterpart: without a meta-file system, per-load
///   configuration belongs in the format value itself — construct the
///   format with fields instead of implementing a settings type.
/// - bevy's `Reader` + `LoadContext` collapse to plain `&[u8]`: no
///   streaming, no sub-assets, which is what keeps `parse` pure and
///   unit-testable.
///
/// # Example
///
/// ```
/// use konnektoren_core::assets::AssetFormat;
///
/// /// Per-format configuration lives on the value (bevy would use
/// /// `type Settings`): this format can be lenient about trailing noise.
/// struct LineCountFormat {
///     skip_blank: bool,
/// }
///
/// impl AssetFormat for LineCountFormat {
///     type Asset = usize;
///     type Error = std::str::Utf8Error;
///
///     fn extensions(&self) -> &[&str] {
///         &["txt"]
///     }
///
///     fn parse(&self, bytes: &[u8]) -> Result<Self::Asset, Self::Error> {
///         let text = std::str::from_utf8(bytes)?;
///         Ok(text
///             .lines()
///             .filter(|line| !self.skip_blank || !line.trim().is_empty())
///             .count())
///     }
/// }
///
/// let format = LineCountFormat { skip_blank: true };
/// assert_eq!(format.parse(b"a\n\nb\n").unwrap(), 2);
/// ```
pub trait AssetFormat {
    /// The typed asset this format produces.
    type Asset;

    /// The format's own parse error; wrapped into
    /// [`AssetError::Parse`](crate::assets::AssetError::Parse) by
    /// [`AssetSource::load`](crate::assets::AssetSource::load).
    type Error: std::error::Error + Send + Sync + 'static;

    /// File extensions this format handles, without the leading dot
    /// (e.g. `["yml", "yaml"]`).
    ///
    /// Advisory, as in bevy: loading does not enforce them, and callers may
    /// parse files with non-matching extensions. Adapters and future
    /// collection derives use them for routing. Defaults to an empty list —
    /// the format claims no extensions.
    fn extensions(&self) -> &[&str] {
        &[]
    }

    /// Parses raw bytes into the typed asset.
    fn parse(&self, bytes: &[u8]) -> Result<Self::Asset, Self::Error>;
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

    /// Relies on the default `extensions()` implementation.
    struct ByteLenFormat;

    impl AssetFormat for ByteLenFormat {
        type Asset = usize;
        type Error = std::convert::Infallible;

        fn parse(&self, bytes: &[u8]) -> Result<Self::Asset, Self::Error> {
            Ok(bytes.len())
        }
    }

    #[test]
    fn parses_valid_bytes() {
        assert_eq!(Utf8Format.parse(b"hallo").unwrap(), "hallo");
    }

    #[test]
    fn rejects_invalid_bytes() {
        assert!(Utf8Format.parse(&[0xff, 0xfe]).is_err());
    }

    #[test]
    fn reports_extensions() {
        assert_eq!(Utf8Format.extensions(), &["txt"]);
    }

    #[test]
    fn extensions_default_to_empty() {
        assert!(ByteLenFormat.extensions().is_empty());
        assert_eq!(ByteLenFormat.parse(b"abc").unwrap(), 3);
    }
}
