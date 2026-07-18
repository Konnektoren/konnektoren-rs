use konnektoren_core::assets::{AssetError, AssetSource};
use std::{future::Future, pin::Pin, rc::Rc};

#[cfg(feature = "zip")]
use std::{cell::RefCell, collections::HashMap, sync::Arc};

#[cfg(feature = "zip")]
type ZipEntries = Arc<HashMap<String, Vec<u8>>>;
#[cfg(feature = "zip")]
type CachedZipEntries = Rc<RefCell<Option<ZipEntries>>>;

/// Object-safe bridge for composing core's async-by-contract sources.
pub trait SourceLayer {
    fn load_layer_bytes<'a>(
        &'a self,
        path: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, AssetError>> + 'a>>;
}

impl<T: AssetSource> SourceLayer for T {
    fn load_layer_bytes<'a>(
        &'a self,
        path: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, AssetError>> + 'a>> {
        Box::pin(self.load_bytes(path))
    }
}

/// Resolves sources in order, falling through only when a source has no entry.
#[derive(Clone, Default)]
pub struct LayeredSource {
    sources: Vec<Rc<dyn SourceLayer>>,
}
impl LayeredSource {
    pub fn new(sources: Vec<Rc<dyn SourceLayer>>) -> Self {
        Self { sources }
    }
    pub fn push(&mut self, source: Rc<dyn SourceLayer>) {
        self.sources.push(source);
    }
}
impl AssetSource for LayeredSource {
    async fn load_bytes(&self, path: &str) -> Result<Vec<u8>, AssetError> {
        for source in &self.sources {
            match source.load_layer_bytes(path).await {
                Ok(bytes) => return Ok(bytes),
                Err(AssetError::NotFound { .. }) => {}
                Err(error) => return Err(error),
            }
        }
        Err(AssetError::NotFound { path: path.into() })
    }
}

/// Serves entries from a ZIP archive supplied by another source.
#[cfg(feature = "zip")]
#[derive(Clone)]
pub struct ZipSource<S: AssetSource + ?Sized> {
    source: Rc<S>,
    archive_path: String,
    entries: CachedZipEntries,
}
#[cfg(feature = "zip")]
impl<S: AssetSource + ?Sized> ZipSource<S> {
    pub fn new(source: Rc<S>, archive_path: impl Into<String>) -> Self {
        Self {
            source,
            archive_path: archive_path.into(),
            entries: Rc::default(),
        }
    }
    fn read_entries(&self, bytes: Vec<u8>) -> Result<ZipEntries, AssetError> {
        let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes)).map_err(|error| {
            AssetError::Load {
                path: self.archive_path.clone(),
                message: error.to_string(),
            }
        })?;
        let mut entries = HashMap::new();
        for index in 0..archive.len() {
            let mut file = archive.by_index(index).map_err(|error| AssetError::Load {
                path: self.archive_path.clone(),
                message: error.to_string(),
            })?;
            if file.is_dir() {
                continue;
            }
            let mut bytes = Vec::new();
            std::io::Read::read_to_end(&mut file, &mut bytes).map_err(|error| {
                AssetError::Load {
                    path: self.archive_path.clone(),
                    message: error.to_string(),
                }
            })?;
            entries.insert(file.name().to_owned(), bytes);
        }
        Ok(Arc::new(entries))
    }
}
#[cfg(feature = "zip")]
impl<S: AssetSource + ?Sized> AssetSource for ZipSource<S> {
    async fn load_bytes(&self, path: &str) -> Result<Vec<u8>, AssetError> {
        let entries = if let Some(entries) = self.entries.borrow().clone() {
            entries
        } else {
            let bytes = self.source.load_bytes(&self.archive_path).await?;
            let entries = self.read_entries(bytes)?;
            *self.entries.borrow_mut() = Some(entries.clone());
            entries
        };
        entries
            .get(path)
            .cloned()
            .ok_or_else(|| AssetError::NotFound { path: path.into() })
    }
}
