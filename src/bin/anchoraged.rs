use std::fs::File;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
    middleware::{self, Next},
};
use hyper::Request;
use tokio::time::Instant;

use anchorage::{storage, Storage};
use anchorage::blobserver::server;
use tracing::{info, error};

/**
 * This binary runs the blob server.
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

    let blob_routes = server::new_router();
    // Crazy into/from stuff going on here, but declaring the type so we know it's
    // still Router<AppState>
    let blob_router: Router<AppState> = blob_routes.with_state(app_state.clone().into());

    let router = Router::new()
        .route("/healthz", get(healthz))
        .merge(blob_router)
        .with_state(app_state)
        .layer(middleware::from_fn(log_request_response));

    // initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .json()
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
fn store(config: &Config) -> impl Storage {
    match &config.storage {
        StorageConfig::Local { directory } => storage::Local::new(directory.clone()),
    }
}

// AppState is passed around to every handler as the main innards of the service.
#[derive(Clone)]
struct AppState {
    started: Instant,
    store: Arc<dyn Storage + Send + Sync>,
}

// Splitting an AppState into something specific for the server implementations
//
// Ignoring clippy warnings here since I want the server module to be independent of the
// binary's specific types
#[allow(clippy::from_over_into)]
impl Into<server::State> for AppState {
    fn into(self) -> server::State {
        server::State { store: self.store }
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

async fn log_request_response<B>(
    req: Request<B>,
    next: Next<B>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    info!(target: "request received", method = req.method().as_str(), path = req.uri().path());
    let res = next.run(req).await;

    let resp_code = res.status().as_u16();
    // Has to be called different ways since you can't use `event!` without a constant value for level
    if resp_code < 200 || resp_code > 299 {
        info!(code = resp_code, "response");
    } else {
        error!(code = resp_code, "response");
    };

    Ok(res)
}
