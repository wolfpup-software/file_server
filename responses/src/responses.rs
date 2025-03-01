use http_body_util::{BodyExt, Full};
use hyper::body::Incoming as IncomingBody;
use hyper::header::{HeaderValue, CONTENT_TYPE, RANGE};
use hyper::http::{Request, Response};
use hyper::Method;
use hyper::StatusCode;
use std::path::PathBuf;

use crate::content_type::HTML;
use crate::type_flyweight::AvailableEncodings;
// use crate::get_range_response::build_get_range_response_from_filepath;
use crate::get_response::build_get_response;
use crate::head_response::build_head_response;
use crate::last_resort_response::build_last_resort_response;

use crate::type_flyweight::BoxedResponse;

pub const NOT_FOUND_416: &str = "416 requested range not satisfiable";
pub const NOT_FOUND_404: &str = "404 not found";
pub const METHOD_NOT_ALLOWED_405: &str = "405 method not allowed";

pub async fn build_response(
    req: Request<IncomingBody>,
    directory: PathBuf,
    content_encodings: Option<Vec<String>>,
    fallback_404: Option<PathBuf>,
) -> Result<BoxedResponse, hyper::http::Error> {
    match req.method() {
        &Method::HEAD => build_head_response(req, directory, content_encodings).await,
        &Method::GET => build_get_response(req, directory, content_encodings, fallback_404).await,
        _ => build_last_resort_response(StatusCode::METHOD_NOT_ALLOWED, METHOD_NOT_ALLOWED_405),
    }
}
