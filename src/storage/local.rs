use anyhow::Result;

use std::fs::File;
use std::io::Write;
use std::path::Path;

pub struct Local {
    directory: String,
}

impl Local {
    pub fn new(directory: String) -> Self {
        Self { directory }
    }
}

impl crate::blobserver::Store for Local {
    fn get(&self, _hash: &str) -> Result<Vec<u8>> {
        todo!();
    }

    fn put(&self, name: &str, data: Vec<u8>) -> Result<()> {
        // If the file is there, return early
        let path = Path::new(&self.directory).join(name);
        if File::open(&path).is_ok() {
            return Ok(());
        }

        // Otherwise, create the file and write the data to it
        let mut f = File::create(&path)?;
        f.write_all(&data[..])?;

        Ok(())
    }
}
