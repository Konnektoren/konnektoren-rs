use crate::AssetLoadError;
use futures::future::{FutureExt, LocalBoxFuture, Shared};
use konnektoren_core::assets::{AssetFormat, AssetSource};
use std::{
    any::{Any, TypeId},
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
    sync::Arc,
};

type ErasedAsset = Arc<dyn Any + Send + Sync>;
type InFlight = Shared<LocalBoxFuture<'static, Result<ErasedAsset, AssetLoadError>>>;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct CacheKey {
    path: String,
    asset_type: TypeId,
}
#[derive(Default)]
struct State {
    cached: HashMap<CacheKey, ErasedAsset>,
    in_flight: HashMap<CacheKey, InFlight>,
}

/// Bevy-`AssetServer`-inspired cache and request coordinator.
///
/// It is local (`Rc`) because core sources may return non-`Send` browser
/// futures. This keeps the same server usable in CSR and SSR code.
#[derive(Clone)]
pub struct AssetServer<S: AssetSource + ?Sized> {
    source: Rc<S>,
    state: Rc<RefCell<State>>,
}

impl<S: AssetSource + ?Sized> AssetServer<S> {
    pub fn new(source: Rc<S>) -> Self {
        Self {
            source,
            state: Rc::default(),
        }
    }
    pub fn source(&self) -> &Rc<S> {
        &self.source
    }
    pub fn clear(&self) {
        self.state.borrow_mut().cached.clear();
    }
    pub fn invalidate(&self, path: &str) {
        self.state
            .borrow_mut()
            .cached
            .retain(|key, _| key.path != path);
    }

    pub async fn load<F>(
        &self,
        format: F,
        path: impl Into<String>,
    ) -> Result<Arc<F::Asset>, AssetLoadError>
    where
        S: 'static,
        F: AssetFormat + Clone + 'static,
        F::Asset: Send + Sync + 'static,
    {
        let path = path.into();
        let key = CacheKey {
            path: path.clone(),
            asset_type: TypeId::of::<F::Asset>(),
        };
        if let Some(asset) = self.state.borrow().cached.get(&key).cloned() {
            return downcast(asset, &path);
        }
        let future = if let Some(future) = self.state.borrow().in_flight.get(&key).cloned() {
            future
        } else {
            let source = self.source.clone();
            let parse_path = path.clone();
            let future =
                async move {
                    let bytes = source.load_bytes(&parse_path).await.map_err(|error| {
                        AssetLoadError::Load {
                            path: parse_path.clone(),
                            message: error.to_string(),
                        }
                    })?;
                    let asset = format
                        .parse(&bytes)
                        .map_err(|error| AssetLoadError::Parse {
                            path: parse_path,
                            message: error.to_string(),
                        })?;
                    Ok(Arc::new(asset) as ErasedAsset)
                }
                .boxed_local()
                .shared();
            self.state
                .borrow_mut()
                .in_flight
                .insert(key.clone(), future.clone());
            future
        };
        let result = future.await;
        let mut state = self.state.borrow_mut();
        state.in_flight.remove(&key);
        if let Ok(asset) = &result {
            state.cached.insert(key, asset.clone());
        }
        drop(state);
        result.and_then(|asset| downcast(asset, &path))
    }
}

impl<S: AssetSource + Default> Default for AssetServer<S> {
    fn default() -> Self {
        Self::new(Rc::new(S::default()))
    }
}

fn downcast<T: Send + Sync + 'static>(
    asset: ErasedAsset,
    path: &str,
) -> Result<Arc<T>, AssetLoadError> {
    asset
        .downcast::<T>()
        .map_err(|_| AssetLoadError::TypeMismatch {
            path: path.to_owned(),
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use konnektoren_core::assets::EmbeddedSource;
    #[derive(Clone)]
    struct Text;
    impl AssetFormat for Text {
        type Asset = String;
        type Error = std::string::FromUtf8Error;
        fn parse(&self, bytes: &[u8]) -> Result<String, Self::Error> {
            String::from_utf8(bytes.to_vec())
        }
    }
    static ASSETS: EmbeddedSource = EmbeddedSource::new(&[("hello.txt", b"hello")]);
    #[test]
    fn returns_the_same_cached_instance() {
        futures::executor::block_on(async {
            let server = AssetServer::new(Rc::new(ASSETS));
            let first = server.load(Text, "hello.txt").await.unwrap();
            let second = server.load(Text, "hello.txt").await.unwrap();
            assert!(Arc::ptr_eq(&first, &second));
        });
    }
}
