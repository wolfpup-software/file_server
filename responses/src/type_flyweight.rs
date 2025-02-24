use bytes::Bytes;
use http_body_util::combinators::BoxBody;
use hyper::Response;
use std::path::PathBuf;
use tokio::io;

pub type BoxedResponse = Response<BoxBody<Bytes, io::Error>>;

#[derive(Debug)]
pub struct RequestDetails {
    pub method: hyper::http::Method,
    pub path: String,
    pub content_encoding: Option<Vec<String>>,
    pub range: Option<String>,
}

type FallbackFilepaths = Vec<(PathBuf, String, Option<String>)>;

#[derive(Clone, Debug)]
pub struct AvailableEncodings {
    pub gzip: bool,
    pub deflate: bool,
    pub br: bool,
    pub zstd: bool,
}

#[derive(Clone, Debug)]
pub struct ServiceRequirements {
    pub directory: PathBuf,
    pub encodings: AvailableEncodings,
    pub filepath_404s: FallbackFilepaths,
}
