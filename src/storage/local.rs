use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use crate::error::{Error, Kind, WithKind};
use crate::{Node, StorageError};

// Prefixes for the different types of files.
//
// This makes it a bit easier to figure out if what the
// id refers to is a blob or node.
const BLOB_PREFIX: &str = "blob-";
const NODE_PREFIX: &str = "node-";

// Constructs an id from a blob hash with the prefix
fn blob_id(hash: &str) -> String {
    format!("{}{}", BLOB_PREFIX, hash)
}
// Constructs an id from a blob hash with the prefix
fn node_id(hash: &str) -> String {
    format!("{}{}", NODE_PREFIX, hash)
}

/// An implementation of a blobstore that is contained in a single,
/// local directory.
pub struct Local {
    directory: String,
}

impl Local {
    pub fn new(directory: String) -> Self {
        Self { directory }
    }
}

impl crate::Storage for Local {
    fn get(&self, hash: &str) -> Result<Vec<u8>, StorageError> {
        let path = Path::new(&self.directory).join(blob_id(hash));

        tracing::debug!("path: {}", path.display());

        let mut buf = vec![];
        let mut f = File::open(path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                return StorageError::NotFound;
            }

            StorageError::IO(e.to_string())
        })?;
        f.read_to_end(&mut buf)?;

        Ok(buf)
    }

    fn put(&self, hash: &str, data: Vec<u8>) -> Result<(), StorageError> {
        // If the file is there, return early
        let path = Path::new(&self.directory).join(blob_id(hash));
        if File::open(&path).is_ok() {
            return Ok(());
        }

        // Otherwise, create the file and write the data to it
        let mut f = File::create(&path)?;
        f.write_all(&data[..])?;

        Ok(())
    }
}

impl From<std::io::Error> for StorageError {
    fn from(value: std::io::Error) -> Self {
        Self::IO(value.to_string())
    }
}

impl crate::NodeStore for Local {
    fn get(&self, hash: &str) -> Result<Node, Error> {
        let path = Path::new(&self.directory).join(node_id(hash));

        let f = File::open(path).map_err(|e| {
            let kind = if e.kind() == std::io::ErrorKind::NotFound {
                Kind::NotFound
            } else {
                Kind::Internal
            };

            Error::from_err("error finding node", e, kind)
        })?;

        serde_json::from_reader(f)
            .map_err(|e| Error::from_err("error decoding json", e, Kind::Internal))
    }

    fn put(&self, hash: &str, node: &Node) -> Result<(), Error> {
        // If the file is there, return early
        let path = Path::new(&self.directory).join(node_id(hash));
        if File::open(&path).is_ok() {
            return Ok(());
        }

        let f = File::create(&path).with_kind("error creating file", Kind::Internal)?;
        serde_json::to_writer_pretty(f, node).with_kind("error writing json", Kind::Internal)?;

        // Otherwise, create the file and write the data to it

        Ok(())
    }
}
