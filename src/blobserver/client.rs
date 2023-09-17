use base64::{engine::general_purpose, Engine as _};
use serde::de::DeserializeOwned;
use std::result::Result;

use crate::blobserver::server;
use crate::error::Error;
use crate::Node;

use super::server::CreateNodeRequest;

pub struct Client {
    remote: String,
    client: reqwest::Client,
}

impl Default for Client {
    fn default() -> Self {
        Self {
            remote: String::from("http://localhost:4444"),
            client: reqwest::Client::new(),
        }
    }
}

/// Handles the response from the server, switching between
/// the given struct to decode to vs the error struct when
/// a non-200 code is received.
///
/// It returns a result to make it match the handler return type.
async fn handle_resp<T: DeserializeOwned>(resp: reqwest::Response) -> Result<T, Error> {
    // Non-200 should unmarshal to an error
    let status = resp.status();
    if !status.is_success() {
        return Err(resp.json().await?);
    }

    Ok(resp.json().await?)
}

impl Client {
    /// Calls to the server to create a new blob.
    ///
    /// The request is encoded base64 for safe transfer.
    /// If the blob already exists, this is an idempotent response:
    /// the same struct will come back with the same ID.
    pub async fn put_blob(&self, data: &[u8]) -> Result<server::CreateBlobResponse, Error> {
        let body = server::CreateBlobRequest {
            data: general_purpose::STANDARD_NO_PAD.encode(data),
        };
        let path = format!("{}/blob", self.remote);
        handle_resp(self.client.put(path).json(&body).send().await?).await
    }

    /// Calls the server to retrieve a blob.
    ///
    /// If it's not found, expect a 404 status error.
    pub async fn get_blob(&self, hash: &str) -> Result<server::BlobResponse, Error> {
        let path = format!("{}/blob/{}", self.remote, hash);
        handle_resp(self.client.get(path).send().await?).await
    }

    /// Calls the server to create a node.
    pub async fn create_node(&self, node: CreateNodeRequest) -> Result<Node, Error> {
        let path = format!("{}/node", self.remote);
        handle_resp(self.client.post(path).json(&node).send().await?).await
    }
}
