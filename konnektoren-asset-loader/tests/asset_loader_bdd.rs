use cucumber::{World, given, then, when};
use konnektoren_asset_loader::{AssetCollection, AssetFormat, AssetServer};
use konnektoren_core::assets::EmbeddedSource;
use std::{fmt, rc::Rc, sync::Arc};

static ASSETS: EmbeddedSource = EmbeddedSource::new(&[("texts/greeting.txt", b"Hallo")]);

#[derive(Clone)]
struct TextFormat;

impl AssetFormat for TextFormat {
    type Asset = String;
    type Error = std::string::FromUtf8Error;

    fn parse(&self, bytes: &[u8]) -> Result<Self::Asset, Self::Error> {
        String::from_utf8(bytes.to_vec())
    }
}

#[derive(AssetCollection)]
#[asset(folder = "texts")]
struct TextAssets;

#[derive(World)]
struct AssetWorld {
    server: AssetServer<EmbeddedSource>,
    loaded: Option<Arc<String>>,
    requests_share_asset: bool,
}

impl fmt::Debug for AssetWorld {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AssetWorld")
            .field("loaded", &self.loaded)
            .field("requests_share_asset", &self.requests_share_asset)
            .finish()
    }
}

impl Default for AssetWorld {
    fn default() -> Self {
        Self {
            server: AssetServer::new(Rc::new(ASSETS)),
            loaded: None,
            requests_share_asset: false,
        }
    }
}

#[given(expr = "an application has a greeting asset")]
async fn application_has_a_greeting_asset(_world: &mut AssetWorld) {}

#[when(expr = "the greeting is loaded from its collection")]
async fn greeting_is_loaded_from_its_collection(world: &mut AssetWorld) {
    world.loaded = match TextAssets::load(&world.server, TextFormat, "greeting.txt").await {
        Ok(greeting) => Some(greeting),
        Err(error) => panic!("the declared greeting should load: {error}"),
    };
}

#[when(expr = "the greeting is loaded by its runtime path")]
async fn greeting_is_loaded_by_its_runtime_path(world: &mut AssetWorld) {
    let path = String::from("texts/greeting.txt");
    world.loaded = match world.server.load(TextFormat, path).await {
        Ok(greeting) => Some(greeting),
        Err(error) => panic!("the runtime greeting should load: {error}"),
    };
}

#[when(expr = "the greeting is requested twice")]
async fn greeting_is_requested_twice(world: &mut AssetWorld) {
    let first = match world.server.load(TextFormat, "texts/greeting.txt").await {
        Ok(greeting) => greeting,
        Err(error) => panic!("the first greeting request should load: {error}"),
    };
    let second = match world.server.load(TextFormat, "texts/greeting.txt").await {
        Ok(greeting) => greeting,
        Err(error) => panic!("the second greeting request should load: {error}"),
    };
    world.requests_share_asset = Arc::ptr_eq(&first, &second);
}

#[then(expr = "the loaded greeting should be {string}")]
async fn loaded_greeting_should_be(world: &mut AssetWorld, greeting: String) {
    assert_eq!(
        world.loaded.as_deref().map(String::as_str),
        Some(greeting.as_str())
    );
}

#[then(expr = "both requests should share the loaded greeting")]
async fn requests_should_share_the_loaded_greeting(world: &mut AssetWorld) {
    assert!(world.requests_share_asset);
}

#[tokio::main]
async fn main() {
    AssetWorld::cucumber()
        .max_concurrent_scenarios(1)
        .run_and_exit("tests/features")
        .await;
}
