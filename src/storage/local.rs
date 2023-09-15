use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;

use crate::error::{Error, Kind};

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

impl crate::blob::Store for Local {
    fn get(&self, hash: &str) -> Result<Vec<u8>, Error> {
        let path = Path::new(&self.directory).join(hash);

        tracing::debug!("path: {}", path.display());

        let mut buf = vec![];
        let mut f = File::open(path).map_err(|e| {
            let kind = match e.kind() {
                io::ErrorKind::NotFound => Kind::NotFound,
                _ => Kind::Internal,
            };

            Error::from_err("error opening file", Box::new(e), kind)
        })?;
        f.read_to_end(&mut buf)?;

        Ok(buf)
    }

    fn put(&self, hash: &str, data: Vec<u8>) -> Result<(), Error> {
        // If the file is there, return early
        let path = Path::new(&self.directory).join(hash);
        if File::open(&path).is_ok() {
            return Ok(());
        }

        // Otherwise, create the file and write the data to it
        let mut f = File::create(&path)?;
        f.write_all(&data[..])?;

        Ok(())
    }
}
