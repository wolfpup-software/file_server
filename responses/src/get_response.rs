use futures_util::TryStreamExt;
use http_body_util::Full;
use http_body_util::{BodyExt, StreamBody};
use hyper::body::Frame;
use hyper::body::Incoming as IncomingBody;
use hyper::header::ACCEPT_RANGES;
use hyper::header::{CONTENT_ENCODING, CONTENT_LENGTH, CONTENT_TYPE};
use hyper::http::Request;
use hyper::http::Response;
use hyper::StatusCode;
use std::path::Path;
use std::path::PathBuf;
use tokio::fs::File;
use tokio_util::io::ReaderStream;

use crate::content_type::get_content_type;
use crate::content_type::HTML;
use crate::last_resort_response::build_last_resort_response;
use crate::response_paths::get_path_from_request_url;
use crate::type_flyweight::BoxedResponse;

pub const NOT_FOUND_416: &str = "416 requested range not satisfiable";
pub const NOT_FOUND_404: &str = "404 not found";
pub const METHOD_NOT_ALLOWED_405: &str = "405 method not allowed";

pub async fn build_get_response(
    req: Request<IncomingBody>,
    directory: PathBuf,
    content_encodings: Option<Vec<String>>,
    fallback_404: Option<PathBuf>,
) -> Result<BoxedResponse, hyper::http::Error> {
    let filepath = match get_path_from_request_url(&req, &directory).await {
        Some(fp) => fp,
        _ => return build_last_resort_response(StatusCode::NOT_FOUND, NOT_FOUND_404),
    };

    let content_type = get_content_type(&filepath);

    // origin target
    if let Some(res) = compose_get_response(&filepath, content_type, None).await {
        return res;
    }

    build_last_resort_response(StatusCode::NOT_FOUND, NOT_FOUND_404)
}

async fn compose_get_response(
    filepath: &PathBuf,
    content_type: &str,
    content_encoding: Option<String>,
) -> Option<Result<BoxedResponse, hyper::http::Error>> {
    let file = match File::open(filepath).await {
        Ok(m) => m,
        _ => return None,
    };

    let metadata = match file.metadata().await {
        Ok(m) => m,
        _ => return None,
    };

    let mut builder = Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, content_type)
        .header(CONTENT_LENGTH, metadata.len());

    if let Some(enc) = content_encoding {
        builder = builder.header(CONTENT_ENCODING, enc);
    }

    // https://github.com/hyperium/hyper/blob/master/examples/send_file.rs
    let reader_stream = ReaderStream::new(file);
    let stream_body = StreamBody::new(reader_stream.map_ok(Frame::data));
    let boxed_body = stream_body.boxed();

    return Some(builder.body(boxed_body));
}
