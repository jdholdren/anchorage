mod client;
use anyhow::Result as anyResult;
pub use client::*;

use crate::error::{Error, Kind, WithReqContext};
use crate::ReqContext;
use std::result::Result;

use serde::{Deserialize, Serialize};

use std::sync::Arc;

use axum::extract::Json as exJson;
use axum::extract::State as exState;
use axum::{routing::post, Json, Router};

use base64::{engine::general_purpose, Engine as _};
use sha256::digest;

// A store is something to manage blobs of files
pub trait Store {
    fn get(&self, name: &str) -> anyResult<Vec<u8>>;
    fn put(&self, name: &str, data: Vec<u8>) -> anyResult<()>;
}

#[derive(Clone)]
pub struct State {
    pub store: Arc<dyn Store + Send + Sync>,
}

pub fn new_router() -> Router<State> {
    Router::new().route("/blob", post(create_blob))
}

#[derive(Deserialize)]
pub struct CreateBlobRequest {
    data: String,
}

impl CreateBlobRequest {
    fn validate(&self) -> Result<(), &'static str> {
        if self.data.is_empty() {
            return Err("data is empty");
        }

        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
pub struct CreateBlobResponse {
    created: String,
}

// Endpoint for ingesting a
async fn create_blob(
    exState(state): exState<State>,
    exJson(body): exJson<CreateBlobRequest>,
) -> Result<Json<CreateBlobResponse>, Error> {
    let ctx = ReqContext {
        user: String::from("unknown"),
        op: String::from("create_blob"),
    };
    body.validate().with_ctx(&ctx, Kind::BadRequest)?;

    // Decode the base64 encoded data
    let data = general_purpose::STANDARD_NO_PAD
        .decode(body.data)
        .with_ctx(&ctx, Kind::BadRequest)?;

    // The name of the file will be the hash of the contents
    let hash = digest(data.as_slice());

    // Store it in the blob store
    state
        .store
        .put(&hash, data)
        .with_ctx(&ctx, Kind::Internal)?;

    Ok(Json(CreateBlobResponse { created: hash }))
}