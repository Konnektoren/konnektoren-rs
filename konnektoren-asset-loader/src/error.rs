#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub enum AssetLoadError {
    #[error("failed to load {path}: {message}")]
    Load { path: String, message: String },
    #[error("failed to parse {path}: {message}")]
    Parse { path: String, message: String },
    #[error("asset type mismatch in cache: {path}")]
    TypeMismatch { path: String },
}
