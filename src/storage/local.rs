use crate::error::Error;
use std::result::Result;

pub struct Local {
    directory: String,
}

impl crate::storage::Store for Local {
    fn get(&self, name: &str) -> Result<Vec<u8>, Error> {
        todo!();
    }

    fn put(&self, name: &str, data: Vec<u8>) -> Result<(), Error> {
        todo!();
    }
}
