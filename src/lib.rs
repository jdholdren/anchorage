pub mod blobserver;
pub mod chunk;
pub mod error;
pub mod storage;

pub struct ReqContext {
    user: String,
    op: String,
}
