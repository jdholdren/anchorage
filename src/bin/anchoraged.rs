use std::fs::File;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use axum::{extract::State, response::IntoResponse, routing::get, Json, Router};

use tokio::time::Instant;
use tower_http::trace::TraceLayer;

use anchorage::blobserver;
use anchorage::storage;

/**
 * This binary runs the server
 **/

#[derive(Debug, Deserialize)]
struct Config {
    port: u16,
    storage: StorageConfig,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum StorageConfig {
    Local { directory: String },
}

#[tokio::main]
async fn main() {
    let config = config();
    let store = store(&config);

    let app_state = AppState {
        started: Instant::now(),
        store: Arc::new(store),
    };

    let blob_routes = blobserver::new_router();
    // Crazy into/from stuff going on here, but declaring the type so we know it's
    // still Router<AppState>
    let blob_router: Router<AppState> = blob_routes.with_state(app_state.clone().into());

    let router = Router::new()
        .route("/healthz", get(healthz))
        .nest("/blobstore", blob_router)
        .with_state(app_state);

    // initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let formatted = format!("0.0.0.0:{}", config.port);
    println!("listening on: {}", formatted);

    axum::Server::bind(&formatted.parse().unwrap())
        .serve(router.into_make_service())
        .await
        .unwrap();
}

fn config() -> Config {
    // Load some env config
    let Ok(config_path) = std::env::var("CONFIG_PATH") else {
        // Just return a default config
        return Config {
            port: 4444,
            storage: StorageConfig::Local { directory: String::from("./file_store") },
        }
    };

    File::open(config_path)
        .map_err(|err| err.to_string())
        .and_then(|f| serde_yaml::from_reader(f).map_err(|err| err.to_string()))
        .unwrap()
}

// Configures a new blob store from what the config says
fn store(config: &Config) -> impl blobserver::Store {
    match &config.storage {
        StorageConfig::Local { directory } => storage::Local::new(directory.clone()),
    }
}

#[derive(Clone)]
struct AppState {
    started: Instant,
    store: Arc<dyn blobserver::Store + Send + Sync>,
}

// Splitting an AppState into something specific for the server implementations
//
// Ignoring clippy warnings here since I want the server module to be independent of the
// binary's specific types
#[allow(clippy::from_over_into)]
impl Into<blobserver::State> for AppState {
    fn into(self) -> blobserver::State {
        blobserver::State { store: self.store }
    }
}

#[derive(Serialize)]
struct HealthzResponse {
    uptime_secs: u64,
}

async fn healthz(State(state): State<AppState>) -> impl IntoResponse {
    Json(HealthzResponse {
        uptime_secs: state.started.elapsed().as_secs(),
    })
}
