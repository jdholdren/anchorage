use std::fs::File;
use std::sync::Arc;

use axum::middleware::{self, Next};
use serde::{Deserialize, Serialize};

use axum::{
    body::{Body, Bytes},
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use hyper::Request;
use tokio::time::Instant;

use anchorage::blobserver;
use anchorage::storage;

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

    let blob_routes = blobserver::new_router();
    // Crazy into/from stuff going on here, but declaring the type so we know it's
    // still Router<AppState>
    let blob_router: Router<AppState> = blob_routes.with_state(app_state.clone().into());

    let router = Router::new()
        .route("/healthz", get(healthz))
        .merge(blob_router)
        .with_state(app_state)
        .layer(middleware::from_fn(print_request_response));

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

async fn print_request_response<B>(
    req: Request<B>,
    next: Next<B>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let res = next.run(req).await;

    let (parts, body) = res.into_parts();
    let bytes = buffer_and_print("response", body).await?;
    let res = Response::from_parts(parts, Body::from(bytes));

    println!("response code: {}", res.status());

    Ok(res)
}

async fn buffer_and_print<B>(direction: &str, body: B) -> Result<Bytes, (StatusCode, String)>
where
    B: axum::body::HttpBody<Data = Bytes>,
    B::Error: std::fmt::Display,
{
    let bytes = match hyper::body::to_bytes(body).await {
        Ok(bytes) => bytes,
        Err(err) => {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("failed to read {} body: {}", direction, err),
            ));
        }
    };

    if let Ok(body) = std::str::from_utf8(&bytes) {
        tracing::debug!("{} body = {:?}", direction, body);
    }

    Ok(bytes)
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

// AppState is passed around to every handler as the main innards of the service.
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
