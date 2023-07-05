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

impl crate::blobserver::Store for Local {
    fn get(&self, _hash: &str) -> Result<Vec<u8>, Error> {
        todo!();
    }

    fn put(&self, _name: &str, _data: Vec<u8>) -> Result<(), Error> {
        todo!();
    }
}
