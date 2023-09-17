use std::sync::Arc;
use std::{fmt::Debug, result::Result};

use serde::{Deserialize, Serialize};

use axum::{
    extract::{DefaultBodyLimit, Json as exJson, Path, State as exState},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use hyper::StatusCode;

use crate::{
    error::{Error, Kind},
    Storage,
};

use base64::{engine::general_purpose, Engine as _};
use sha256::digest;

#[derive(Clone)]
pub struct State {
    pub store: Arc<dyn Storage + Send + Sync>,
}

pub fn new_router() -> Router<State> {
    Router::new()
        .route("/blob", post(create_blob))
        .route("/blob/:hash", get(fetch_blob))
        .layer(DefaultBodyLimit::max(1024 * 1024 * 11)) // 11MB
}

/// CreateBlobRequest holds the data to be stored by the server.
#[derive(Serialize, Deserialize)]
pub struct CreateBlobRequest {
    pub data: String,
}

impl CreateBlobRequest {
    // Performs validation on the request, namely
    // that the data is not empty.
    fn validate(&self) -> Result<(), &'static str> {
        if self.data.is_empty() {
            return Err("data is empty");
        }

        Ok(())
    }
}

/// Returned on a successful call to store a blob. It contains
/// the hash that was inserted.
#[derive(Serialize, Deserialize)]
pub struct CreateBlobResponse {
    pub created: String,
}

// Endpoint for ingesting a blob
async fn create_blob(
    exState(state): exState<State>,
    exJson(body): exJson<CreateBlobRequest>,
) -> Result<Json<CreateBlobResponse>, Error> {
    body.validate()
        .map_err(|e| Error::from_msg(e, Kind::BadRequest))?;

    // Decode the base64 encoded data
    let data = general_purpose::STANDARD_NO_PAD
        .decode(body.data)
        .map_err(|e| Error::from_err("error decoding body", e, Kind::BadRequest))?;

    // The name of the file will be the hash of the contents
    let id = format!("sha256-{}", digest(data.as_slice()));

    // Store it in the blob store
    state
        .store
        .put(&id, data)
        .map_err(|e| Error::from_err("error storing blob", e, Kind::BadRequest))?;

    Ok(Json(CreateBlobResponse { created: id }))
}

#[derive(Serialize, Deserialize)]
pub struct BlobResponse {
    pub contents: String,
}

impl Debug for BlobResponse {
    fn fmt(&self, w: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Send it in plain text if it it's utf8
        let bytes = general_purpose::STANDARD_NO_PAD
            .decode(&self.contents)
            .unwrap();
        if let Ok(utf_safe) = String::from_utf8(bytes) {
            return write!(w, "{}", &utf_safe);
        }

        write!(w, "{}", self.contents)
    }
}

// Endpoint for fetching a stored blob
async fn fetch_blob(
    Path(hash): Path<String>,
    exState(state): exState<State>,
) -> Result<impl IntoResponse, Error> {
    let data_res = state
        .store
        .get(&hash)
        .map_err(|e| Error::from_err("error finding blob", e, Kind::NotFound))?;

    // Decode the base64 encoded data
    let data = general_purpose::STANDARD_NO_PAD.encode(data_res);

    Ok((StatusCode::CREATED, Json(BlobResponse { contents: data })))
}
