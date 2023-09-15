pub mod chunk;
pub mod error;
pub mod storage;
pub mod blob;
pub mod blobserver;

#[derive(Debug)]
pub struct ReqContext {
    pub user: String,
    pub op: String,
}

// The types representing the core ideas of the project
