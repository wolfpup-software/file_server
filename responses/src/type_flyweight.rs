use bytes::Bytes;
use http_body_util::combinators::BoxBody;
use hyper::Response;
use std::path::PathBuf;
use tokio::io;

pub type BoxedResponse = Response<BoxBody<Bytes, io::Error>>;

#[derive(Clone, Debug)]
pub struct AvailableEncodings {
    pub gzip: bool,
    pub deflate: bool,
    pub br: bool,
    pub zstd: bool,
}
