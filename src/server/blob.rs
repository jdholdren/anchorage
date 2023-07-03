use crate::error::Error;
use std::result::Result;

use anyhow::anyhow;
use serde::{Deserialize, Serialize};

use std::sync::Arc;

use axum::extract::Json as exJson;
use axum::extract::State as exState;
use axum::{routing::post, Json, Router};

// A store is something to manage blobs of files
pub trait Store {
    fn get(&self, name: &str) -> Result<Vec<u8>, Error>;
    fn put(&self, name: &str, data: Vec<u8>) -> Result<(), Error>;
}

#[derive(Clone)]
pub struct ServerState {
    pub store: Arc<dyn Store + Send + Sync>,
}

pub fn new_router() -> Router<ServerState> {
    Router::new().route("/blob", post(create_blob))
}

#[derive(Deserialize)]
pub struct CreateBlobRequest {
    data: String,
}

impl CreateBlobRequest {
    fn validate(&self) -> Result<(), Error> {
        let mut err = Error {
            user: "unknown".to_string(),
            op: "create_blob",
            kind: crate::error::Kind::BadRequest,
            err: Box::new(anyhow!("must provide data")),
        };

        Err(err)
    }
}

#[derive(Serialize, Deserialize)]
pub struct CreateBlobResponse {}

// Endpoint for ingesting a
async fn create_blob(
    exState(_server_state): exState<ServerState>,
    exJson(body): exJson<CreateBlobRequest>,
) -> Result<Json<CreateBlobResponse>, Error> {
    body.validate()?;

    Ok(Json(CreateBlobResponse {}))
}
