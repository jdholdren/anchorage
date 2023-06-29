use std::error::Error;
use std::fs::File;

use serde::{Deserialize, Serialize};

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use tokio::time::Instant;

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

    let blob_router = create_blob_router();

    // initialize tracing
    tracing_subscriber::fmt::init();

    let formatted = format!("0.0.0.0:{}", config.port);
    println!("listening on: {}", formatted);

    axum::Server::bind(&formatted.parse().unwrap())
        .serve(blob_router.into_make_service())
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

#[derive(Clone)]
struct AppState {
    started: Instant,
}

fn create_blob_router() -> Router {
    Router::new()
        // `GET /` goes to `root`
        .route("/healthz", get(healthz))
        .with_state(AppState {
            started: Instant::now(),
        })
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
