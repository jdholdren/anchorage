use crate::error::Error;

/// Internal representation of a node.
pub struct Node {
    pub id: String,
}

// A store is something to manage blobs of files
pub trait Store {
    fn get(&self, name: &str) -> Result<Vec<u8>, Error>;
    fn put(&self, name: &str, data: Vec<u8>) -> Result<(), Error>;
}

pub struct Service {
}

impl Service {
}
