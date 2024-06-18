use futures_util::TryStreamExt;
use http_body_util::{combinators::BoxBody, BodyExt, Full, StreamBody};
use hyper::body::{Frame, Incoming as IncomingBody};
use hyper::header::{HeaderValue, ACCEPT_ENCODING, CONTENT_ENCODING, CONTENT_TYPE};
use hyper::http::{Request, Response};
use hyper::StatusCode;
use std::{io, path};
use tokio::fs::File;
use tokio_util::io::ReaderStream;

use crate::content_and_encoding_type::get_content_type;

const FWD_SLASH: &str = "/";
const INDEX: &str = "index.html";
const INTERNAL_SERVER_ERROR: &str = "500 internal server error";
const HTML: &str = "text/html; charset=utf-8";

pub type BoxedResponse = Response<BoxBody<bytes::Bytes, io::Error>>;

pub fn get_pathbuff_from_request(
    dir: &path::Path,
    req: &Request<IncomingBody>,
) -> (Option<path::PathBuf>, Option<path::PathBuf>, Option<String>) {
    // get path and strip initial forward slash
    let uri_path = req.uri().path();
    let mut path = match uri_path.strip_prefix(FWD_SLASH) {
        Some(p) => dir.join(p),
        _ => dir.join(uri_path),
    };

    // flatten path (ie ../../)
    if let Some(file_name_str) = path.file_name() {
        path = path::PathBuf::from(file_name_str);
    }

    // confirm path resides in directory
    if !path.starts_with(dir) {
        // and exists?
        return (None, None, None);
    }

    // look for index.html if directory
    if path.is_dir() {
        path = path.join(INDEX);
    }

    if let Some(encoding_value) = req.headers().get(ACCEPT_ENCODING) {
        if let Ok(encoding_str) = encoding_value.to_str() {
            for encoding in encoding_str.split(",") {
                let trimmed = encoding.trim();
                if let Some(ext) = get_ext(trimmed) {
                    let encoding_path = path.join(ext);

                    if let Ok(exists) = encoding_path.try_exists() {
                        if exists {
                            return (Some(path), Some(encoding_path), Some(trimmed.to_string()));
                        }
                    }
                }
            }
        }
    };

    // otherwise serve file
    if let Ok(exists) = path.try_exists() {
        if exists {
            return (Some(path), None, None);
        }
    }

    (None, None, None)
}

fn get_ext(encoding: &str) -> Option<&str> {
    match encoding {
        "gzip" => Some(".gz"),
        "zstd" => Some(".zst"),
        "br" => Some(".br"),
        "deflate" => Some(".zz"),
        _ => None,
    }
}

pub async fn build_response(
    req_path: path::PathBuf,
    encoding_path: Option<path::PathBuf>,
    encoding: Option<String>,
) -> Result<BoxedResponse, hyper::http::Error> {
    let (path, enc) = match (encoding_path, encoding) {
        (Some(enc_path), Some(enc)) => (enc_path, Some(enc)),
        _ => (req_path.clone(), None),
    };

    match File::open(&path).await {
        Ok(file) => {
            // https://github.com/hyperium/hyper/blob/master/examples/send_file.rs

            let content_type = get_content_type(&req_path);

            let reader_stream = ReaderStream::new(file);
            let stream_body = StreamBody::new(reader_stream.map_ok(Frame::data));
            let boxed_body = stream_body.boxed();

            if let Some(enc_type) = enc {
                return Response::builder()
                    .status(StatusCode::OK)
                    .header(CONTENT_TYPE, content_type)
                    .header(CONTENT_ENCODING, enc_type)
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
