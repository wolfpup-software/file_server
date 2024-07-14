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
    let source_dir = match path::absolute(dir) {
        Ok(sdf) => sdf,
        _ => return (None, None, None),
    };

    // asbolute the source_dir
    // flatten path (ie ../../)
    let uri_path = req.uri().path();
    let mut target_path = match uri_path.strip_prefix(FWD_SLASH) {
        Some(p) => source_dir.join(p),
        _ => source_dir.join(uri_path),
    };
    // look for index.html if directory
    if target_path.is_dir() {
        target_path = target_path.join(INDEX);
    }
    target_path = match path::absolute(target_path) {
        Ok(sdf) => sdf,
        _ => return (None, None, None),
    };

    // confirm path resides in directory
    if !target_path.starts_with(source_dir) {
        return (None, None, None);
    }

    // check if encoded file exists
    if let Ok(exists) = target_path.try_exists() {
        if !exists {
            return (None, None, None);
        }
    }

    let (encoding_path, trimmed) = get_enc_path(&target_path, req.headers().get(ACCEPT_ENCODING));

    // otherwise serve file
    (Some(target_path), encoding_path, trimmed)
}

// target path must be absolute for this to work
fn get_enc_path(
    target_path: &path::PathBuf,
    encoding_header: Option<&HeaderValue>,
) -> (Option<path::PathBuf>, Option<String>) {
    let header = match encoding_header {
        Some(enc) => enc,
        _ => return (None, None),
    };

    let encoding_str = match header.to_str() {
        Ok(s) => s,
        _ => return (None, None),
    };

    for encoding in encoding_str.split(",") {
        let trimmed = encoding.trim();
        if let Some(ext) = get_ext(trimmed) {
            let encoding_path = target_path.join(ext);

            if let Ok(exists) = encoding_path.try_exists() {
                if exists {
                    return (Some(encoding_path), Some(trimmed.to_string()));
                }
            }
        }
    }

    (None, None)
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
    req_path: Option<path::PathBuf>,
    encoding_path: Option<path::PathBuf>,
    encoding: Option<String>,
) -> Result<BoxedResponse, hyper::http::Error> {
    let rpath = match req_path {
        Some(p) => p,
        _ => return create_error_response(&StatusCode::NOT_FOUND, "404 not found"),
    };

    let content_type = get_content_type(&rpath);
    let (path, enc) = match (encoding_path, encoding) {
        (Some(enc_path), Some(enc)) => (enc_path, Some(enc)),
        _ => (rpath.clone(), None),
    };

    match File::open(&path).await {
        Ok(file) => {
            let mut builder = Response::builder()
                .status(StatusCode::OK)
                .header(CONTENT_TYPE, content_type);

            if let Some(enc_type) = enc {
                builder = builder.header(CONTENT_ENCODING, enc_type);
            }

            // https://github.com/hyperium/hyper/blob/master/examples/send_file.rs
            let reader_stream = ReaderStream::new(file);
            let stream_body = StreamBody::new(reader_stream.map_ok(Frame::data));
            let boxed_body = stream_body.boxed();

            builder.body(boxed_body)
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
