pub mod blobserver;
pub mod chunk;
pub mod error;
pub mod storage;

use error::Error;

// The types representing the core ideas of the project

#[derive(Debug)]
pub enum StorageError {
    NotFound, // The blob being stored could not be located
    IO(String),
}

impl std::error::Error for StorageError {}
impl std::fmt::Display for StorageError {
    fn fmt(&self, w: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(w, "{:?}", self)
    }
}

// Storages manage blobs bytes.
pub trait Storage {
    fn get(&self, id: &str) -> Result<Vec<u8>, StorageError>;
    fn put(&self, id: &str, data: Vec<u8>) -> Result<(), StorageError>;
}

/// Internal representation of a node.
pub struct Node {
    pub id: String,
    // TODO: node type
}

// NodeStore wraps the surface of how nodes are retrieved.
pub trait NodeStore {
    fn get(&self, id: &str) -> Result<Node, Error>;
    fn put(&self, id: &str, data: Vec<u8>) -> Result<(), Error>;
}
