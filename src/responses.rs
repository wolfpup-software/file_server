use futures_util::TryStreamExt;
use http_body_util::{combinators::BoxBody, BodyExt, Full, StreamBody};
use hyper::body::{Frame, Incoming as IncomingBody};
use hyper::header::{HeaderValue, ACCEPT_ENCODING, CONTENT_ENCODING, CONTENT_TYPE};
use hyper::http::{Request, Response};
use hyper::StatusCode;
use std::{io, path};
use tokio::fs::File;
use tokio_util::io::ReaderStream;

use crate::content_and_encoding::{get_content_type, get_encoded_ext};

const FWD_SLASH: &str = "/";
const INDEX: &str = "index.html";
const INTERNAL_SERVER_ERROR: &str = "500 internal server error";
const HTML: &str = "text/html; charset=utf-8";

pub type BoxedResponse = Response<BoxBody<bytes::Bytes, io::Error>>;

// return encoding NONE or encoding YES
pub fn get_pathbuff_from_request(
    dir: &path::Path,
    req: &Request<IncomingBody>,
) -> (Option<path::PathBuf>, Option<path::PathBuf>, Option<String>) {
    // flatten path, convert to absolute (ie remove ../../)
    let source_dir = match path::absolute(dir) {
        Ok(sdf) => sdf,
        _ => return (None, None, None),
    };

    let uri_path = req.uri().path();
    let mut target_path = match uri_path.strip_prefix(FWD_SLASH) {
        Some(p) => source_dir.join(p),
        _ => source_dir.join(uri_path),
    };

    // if directory look for index.html
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

    // check if file exists
    if let Ok(exists) = target_path.try_exists() {
        if !exists {
            return (None, None, None);
        }
    }

    // check if encoded file exists
    let (encoded_path, encoding_type) =
        get_encoded_path(&target_path, req.headers().get(ACCEPT_ENCODING));

    // otherwise serve file
    (Some(target_path), encoded_path, encoding_type)
}

// target path must be absolute for this to work
fn get_encoded_path(
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
        let enc = encoding.trim();
        let encoded_path = match get_encoded_ext(enc) {
            // add_extension is a nightly feature on std::path
            // for now, get path as string, add ext, get path
            Some(ext) => {
                let target_path_os_str = target_path.to_string_lossy();
                let target_edited = target_path_os_str.to_string() + ext;
                path::PathBuf::from(&target_edited)
            },
            _ => continue,
        };

        if let Ok(exists) = encoded_path.try_exists() {
            if exists {
                return (Some(encoded_path), Some(enc.to_string()));
            }
        }
    }

    (None, None)
}

pub async fn build_response(
    req_path: Option<path::PathBuf>,
    encoded_path: Option<path::PathBuf>,
    encoding: Option<String>,
) -> Result<BoxedResponse, hyper::http::Error> {
    let rpath = match req_path {
        Some(p) => p,
        _ => return create_error_response(&StatusCode::NOT_FOUND, "404 not found"),
    };

    let content_type = get_content_type(&rpath);
    let (path, enc) = match (encoded_path, encoding) {
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
