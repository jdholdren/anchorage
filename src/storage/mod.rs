use crate::error::Error;
use std::result::Result;

// A store is something to manage blobs of files
pub trait Store {
    fn get(&self, name: &str) -> Result<Vec<u8>, Error>;
    fn put(&self, name: &str, data: Vec<u8>) -> Result<(), Error>;
}

// Different implementations of a blob store

pub mod local;
