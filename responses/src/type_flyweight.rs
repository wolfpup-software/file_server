use bytes::Bytes;
use http_body_util::combinators::BoxBody;
use hyper::Response;
use tokio::io;

pub type BoxedResponse = Response<BoxBody<Bytes, io::Error>>;
