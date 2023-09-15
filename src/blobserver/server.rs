use std::sync::Arc;
use std::result::Result;

use serde::{Deserialize, Serialize};

use axum::{
    extract::{DefaultBodyLimit, Json as exJson, Path, State as exState, FromRequestParts},
    routing::{get, post},
    Json, Router,
    async_trait,
    http,response::IntoResponse,
};
use hyper::StatusCode;

use crate::error::{Error, Kind};
use crate::ReqContext;
use crate::blob::Service;

use base64::{engine::general_purpose, Engine as _};
use sha256::digest;

// Custom extractor to add context to requests
struct ContextExtractor(ReqContext);

#[async_trait]
impl<S> FromRequestParts<S> for ContextExtractor {
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(
        parts: &mut http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        Ok(ContextExtractor(ReqContext{
            user: String::from("unknown"),
            op: parts.uri.to_string(),
        }))
    }
}


#[derive(Clone)]
pub struct State {
    pub store: Arc<Service>,
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
    ContextExtractor(ctx): ContextExtractor,
    exState(state): exState<State>,
    exJson(body): exJson<CreateBlobRequest>,
) -> Result<Json<CreateBlobResponse>, Error> {
    body.validate()
        .map_err(|e| ctx.e(Some(e.to_string()), Kind::BadRequest))?;

    // Decode the base64 encoded data
    let data = general_purpose::STANDARD_NO_PAD
        .decode(body.data)
        .map_err(|e| ctx.e(Some(e.to_string()), Kind::BadRequest))?;

    // The name of the file will be the hash of the contents
    let hash = digest(data.as_slice());

    // Store it in the blob store
    state
        .store
        .put(&blob_name(&hash), data)
        .map_err(|e| ctx.e(Some(e.to_string()), Kind::Internal))?;

    Ok(Json(CreateBlobResponse { created: hash }))
}

#[derive(Serialize, Deserialize)]
pub struct BlobResponse {
    pub contents: String,
}

// Endpoint for fetching a stored blob
async fn fetch_blob(
    ContextExtractor(ctx): ContextExtractor,
    Path(hash): Path<String>,
    exState(state): exState<State>,
) -> Result<impl IntoResponse, Error> {
    println!("{:?}", ctx);

    let data_res = state.store.get(&blob_name(&hash));
    if let Err(err) = &data_res {
        let kind = match err {
            StorageError::NotFound => Kind::NotFound,
            _ => Kind::Internal,
        };

        return Err(ctx.e(Some(err.to_string()), kind));
    }

    // Decode the base64 encoded data
    let data = general_purpose::STANDARD_NO_PAD.encode(data_res.unwrap());

    Ok((StatusCode::CREATED, Json(BlobResponse { contents: data })))
}

fn blob_name(hash: &str) -> String {
    format!("blob-{}", hash)
}
