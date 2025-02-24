use bytes::Bytes;
use http_body_util::combinators::BoxBody;
use hyper::Response;
use std::path::PathBuf;
use tokio::io;

pub type BoxedResponse = Response<BoxBody<Bytes, io::Error>>;

#[derive(Debug)]
pub struct RequestDetails {
    pub path: String,
    pub content_encoding: Option<Vec<String>>,
    pub range: Option<String>,
}
