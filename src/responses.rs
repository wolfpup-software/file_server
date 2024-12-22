use futures_util::TryStreamExt;
use http_body_util::{combinators::BoxBody, BodyExt, Full, StreamBody};
use hyper::body::{Frame, Incoming as IncomingBody};
use hyper::header::{HeaderValue, ACCEPT_ENCODING, CONTENT_ENCODING, CONTENT_TYPE};
use hyper::http::{Request, Response};
use hyper::StatusCode;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::{io, path};
use tokio::fs::File;
use tokio_util::io::ReaderStream;

use crate::config::Config;
use crate::content_and_encoding::{get_content_type, get_encoded_ext};

const FWD_SLASH: &str = "/";
const INDEX: &str = "index.html";
const INTERNAL_SERVER_ERROR: &str = "500 internal server error";
const HTML: &str = "text/html; charset=utf-8";
const GZIP: &str = "gzip";
const COMPRESS: &str = "compress";
const DEFLATE: &str = "deflate";
const BR: &str = "br";
const ZSTD: &str = "zstd";

pub type BoxedResponse = Response<BoxBody<bytes::Bytes, io::Error>>;

#[derive(Debug)]
pub struct PathDetails {
    path: PathBuf,
    encoding: String,
}

#[derive(Debug)]
pub struct ReqDetails {
    path: PathBuf,
    content_type: String,
    path_details: Vec<PathDetails>,
}

struct EncodingCheck {
    gzip: bool,
    deflate: bool,
    br: bool,
    zstd: bool,
}

pub fn get_encoding_ext(requested_encoding: &str) -> Option<&str> {
    match requested_encoding {
        "gzip" => Some(".gz"),
        "deflate" => Some(".zz"),
        "br" => Some(".br"),
        "zstd" => Some(".zstd"),
        _ => None,
    }
}

fn get_path_from_request_url(dir: &Path, req: &Request<IncomingBody>) -> Option<PathBuf> {
    let uri_path = req.uri().path();
    // no need to strip uri paths?
    let mut target_path = match uri_path.strip_prefix(FWD_SLASH) {
        Some(p) => dir.join(p),
        _ => dir.join(uri_path),
    };

    // if directory look for index.html
    if target_path.is_dir() {
        target_path = target_path.join(INDEX);
    }

    // confirm path resides in directory
    if target_path.starts_with(dir) {
        // target path is 404
        return Some(target_path);
    }

    None
}

pub fn get_encodings(req: &Request<IncomingBody>) -> Vec<String> {
    let mut encodings = Vec::new();

    let accept_encoding_header = match req.headers().get(ACCEPT_ENCODING) {
        Some(enc) => enc,
        _ => return encodings,
    };

    let encoding_str = match accept_encoding_header.to_str() {
        Ok(s) => s,
        _ => return encodings,
    };

    for encoding in encoding_str.split(",") {
        encodings.push(encoding.trim().to_string());
    }

    encodings
}

pub fn get_paths_from_request(config: &Config, req: &Request<IncomingBody>) -> Option<ReqDetails> {
    let mut paths = Vec::new();

    let req_path = match get_path_from_request_url(&config.directory, req) {
        Some(p) => p,
        _ => return None,
    };

    let content_type = get_content_type(&req_path).to_string();
    let encodings = get_encodings(req);
    println!("{:?}", &encodings);

    let ext = match req_path.extension() {
        Some(ext) => ext.to_os_string(),
        _ => OsString::from(""),
    };

    for encoding in encodings {
        let mut req_path_with_enc_ext = &req_path.clone();
        let enc_from_ext = match get_encoded_ext(&encoding) {
            Some(ext) => ext,
            _ => continue,
        };

        let mut path_os_str = req_path.clone().into_os_string();
        path_os_str.push(enc_from_ext);

        let enc_path = path::PathBuf::from(path_os_str);

        paths.push(PathDetails {
            path: enc_path.clone(),
            encoding: encoding.clone(),
        });
    }

    Some(ReqDetails {
        path: req_path.clone(),
        content_type: content_type,
        path_details: paths,
    })
}

pub fn get_path_details_from_request(
    dir: &path::Path,
    req: &Request<IncomingBody>,
) -> (Option<(path::PathBuf, String)>, Option<String>) {
    println!("{:?}", req);
    // flatten path, convert to absolute (ie resolve ../../)

    // let paths = Vec::new();

    let uri_path = req.uri().path();
    // no need to strip uri paths?
    let mut target_path = match uri_path.strip_prefix(FWD_SLASH) {
        Some(p) => dir.join(p),
        _ => dir.join(uri_path),
    };

    // if directory look for index.html
    if target_path.is_dir() {
        target_path = target_path.join(INDEX);
    }

    // confirm path resides in directory
    if !target_path.starts_with(dir) {
        // target path is 404
        return (None, None);
    }

    let content_type = get_content_type(&target_path).to_string();
    let accept_encoding = req.headers().get(ACCEPT_ENCODING);
    // return enoded file if exists
    if let Some((enc_path, enc_type)) = get_encoded_path(&target_path, accept_encoding) {
        return (Some((enc_path, content_type)), Some(enc_type));
    }

    // check if file exists
    if let Ok(exists) = target_path.try_exists() {
        if !exists {
            // target path is 404
            return (None, None);
        }
    }

    (Some((target_path, content_type)), None)
}

// target path must be absolute for this to work
fn get_encoded_path(
    target_path: &path::PathBuf,
    encoding_header: Option<&HeaderValue>,
) -> Option<(path::PathBuf, String)> {
    let header = match encoding_header {
        Some(enc) => enc,
        _ => return None,
    };

    let encoding_str = match header.to_str() {
        Ok(s) => s,
        _ => return None,
    };

    let path_lossy = target_path.to_string_lossy();

    for encoding in encoding_str.split(",") {
        let enc = encoding.trim();
    }

    None
}

pub async fn build_response(
    target_path_and_content_type: Option<(path::PathBuf, String)>,
    encoding_type: Option<String>,
) -> Result<BoxedResponse, hyper::http::Error> {
    let (target_path, content_type) = match target_path_and_content_type {
        Some((target, content)) => (target, content),
        _ => return create_error_response(&StatusCode::NOT_FOUND, "404 not found"),
    };

    // iterate through path options
    // return first option
    match File::open(&target_path).await {
        Ok(file) => {
            let mut builder = Response::builder()
                .status(StatusCode::OK)
                .header(CONTENT_TYPE, content_type);

            if let Some(enc_type) = encoding_type {
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

    // Otherwise serve 404
    // No error response? there shouldn't be an error. Either file exists or not.
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
