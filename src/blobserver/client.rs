use anyhow::Result;

use crate::blobserver::CreatedBlobResponse;

use super::BlobResponse;

pub struct Client {
    remote: String,
    http: reqwest::Client,
}

impl Default for Client {
    fn default() -> Self {
        Self {
            remote: String::from("http://localhost:4444"),
            http: reqwest::Client::new(),
        }
    }
}

impl Client {
    pub fn put_blob(&self, data: &[u8]) -> Result<CreatedBlobResponse> {
        todo!()
    }

    pub async fn get_blob(&self, hash: &str) -> Result<BlobResponse> {
        let path = format!("{}/blobstore/blob/{}", self.remote, hash);
        println!("path: {}", path);

        let resp: BlobResponse = self.http.get(path).send().await?.json().await?;

        Ok(resp)
    }
}
