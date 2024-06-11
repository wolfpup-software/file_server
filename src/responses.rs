use futures_util::TryStreamExt;
use http_body_util::{combinators::BoxBody, BodyExt, Full, StreamBody};
use hyper::body::{Frame, Incoming as IncomingBody};
use hyper::header::{HeaderValue, CONTENT_ENCODING, CONTENT_TYPE};
use hyper::http::{Request, Response};
use hyper::StatusCode;
use std::{io, path};
use tokio::fs::File;
use tokio_util::io::ReaderStream;

use crate::content_and_encoding_type::{get_content_encoding, get_content_type};

const FWD_SLASH: &str = "/";
const INDEX: &str = "index.html";
const INTERNAL_SERVER_ERROR: &str = "500 internal server error";
const HTML: &str = "text/html; charset=utf-8";

pub type BoxedResponse = Response<BoxBody<bytes::Bytes, io::Error>>;

pub fn get_pathbuff_from_request(
    dir: &path::Path,
    req: &Request<IncomingBody>,
) -> Option<path::PathBuf> {
    // get path and strip forward slash
    let uri_path = req.uri().path();
    let mut strip_path = match uri_path.strip_prefix(FWD_SLASH) {
        Some(p) => dir.join(p),
        _ => dir.join(uri_path),
    };

    if strip_path.is_dir() {
        strip_path = strip_path.join(INDEX);
    }

    if let Some(file_name_str) = strip_path.file_name() {
        let path = path::PathBuf::from(file_name_str);
        // confirm canon'd path resides in directory
        if path.starts_with(dir) {
            return Some(path);
        }
    }

    None
}

pub fn create_error_response(
    code: &StatusCode,
    body: &'static str,
) -> Result<BoxedResponse, hyper::http::Error> {
    Response::builder()
        .status(code)
        .header(CONTENT_TYPE, HeaderValue::from_static(HTML))
        .body(
            Full::new(bytes::Bytes::from(body))
                .map_err(|e| match e {})
                .boxed(),
        )
}

pub async fn build_response(path: path::PathBuf) -> Result<BoxedResponse, hyper::http::Error> {
    match File::open(&path).await {
        Ok(file) => {
            // https://github.com/hyperium/hyper/blob/master/examples/send_file.rs

            let content_encoding = get_content_encoding(&path);
            let content_type = get_content_type(&path);

            let reader_stream = ReaderStream::new(file);
            let stream_body = StreamBody::new(reader_stream.map_ok(Frame::data));
            let boxed_body = stream_body.boxed();

            if let Some(encoding) = content_encoding {
                return Response::builder()
                    .status(StatusCode::OK)
                    .header(CONTENT_TYPE, content_type)
                    .header(CONTENT_ENCODING, encoding)
                    .body(boxed_body);
            }

            Response::builder()
                .status(StatusCode::OK)
                .header(CONTENT_TYPE, content_type)
                .body(boxed_body)
        }
        _ => create_error_response(&StatusCode::INTERNAL_SERVER_ERROR, &INTERNAL_SERVER_ERROR),
    }
}
