use crate::error::Error;
use std::result::Result;

pub struct Local {
    directory: String,
}

impl Local {
    pub fn new(directory: String) -> Self {
        Self { directory }
    }
}

impl crate::server::blob::Store for Local {
    fn get(&self, hash: &str) -> Result<Vec<u8>, Error> {
        todo!();
    }

    fn put(&self, name: &str, data: Vec<u8>) -> Result<(), Error> {
        todo!();
    }
}
