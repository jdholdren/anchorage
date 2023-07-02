use crate::error::Error;

use serde::{Deserialize};

use std::sync::Arc;


use axum::extract::State as exState;
use axum::{
    response::IntoResponse, Router,
};

// A store is something to manage blobs of files
pub trait Store {
    fn get(&self, name: &str) -> Result<Vec<u8>, Error>;
    fn put(&self, name: &str, data: Vec<u8>) -> Result<(), Error>;
}

#[derive(Clone)]
pub struct ServerState {
    pub store: Arc<dyn Store + Send + Sync>,
}

#[derive(Deserialize)]
struct CreateBlobBody {
    data: Vec<u8>,
}

pub fn new_router() -> Router<ServerState> {
    Router::new()
}

// Endpoint for ingesting a
async fn create_blob(exState(_server_state): exState<ServerState>) -> impl IntoResponse {
    
}
