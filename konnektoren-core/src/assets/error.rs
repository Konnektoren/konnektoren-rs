/// Errors produced while loading an asset, from any source or format.
///
/// Sources report [`NotFound`](AssetError::NotFound) or
/// [`Load`](AssetError::Load); [`Parse`](AssetError::Parse) wraps the
/// format's own error type once bytes were obtained.
#[derive(Debug, thiserror::Error)]
pub enum AssetError {
    /// The source could not resolve the requested path.
    #[error("asset not found: {path}")]
    NotFound {
        /// The path as requested from the source.
        path: String,
    },

    /// The source found the asset but failed to deliver its bytes
    /// (I/O error, failed fetch, bad HTTP status, …).
    #[error("failed to load {path}: {message}")]
    Load {
        /// The path as requested from the source.
        path: String,
        /// Human-readable failure description.
        message: String,
    },

    /// Bytes were loaded but the format could not parse them.
    #[error("failed to parse {path}: {source}")]
    Parse {
        /// The path whose bytes failed to parse.
        path: String,
        /// The format's error, type-erased.
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

impl AssetError {
    /// Wraps a format error as [`AssetError::Parse`] for the given path.
    pub fn parse(
        path: impl Into<String>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        AssetError::Parse {
            path: path.into(),
            source: Box::new(source),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_includes_path() {
        let err = AssetError::NotFound {
            path: "challenges/level_a1.yml".into(),
        };
        assert_eq!(err.to_string(), "asset not found: challenges/level_a1.yml");
    }

    #[test]
    fn parse_wraps_source_error() {
        let inner = String::from_utf8(vec![0xff]).unwrap_err();
        let err = AssetError::parse("broken.txt", inner);
        assert!(err.to_string().starts_with("failed to parse broken.txt:"));
        assert!(std::error::Error::source(&err).is_some());
    }
}
