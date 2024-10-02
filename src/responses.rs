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

// req_details_struct
struct ReqDetails {
    accept_encoding_header: String,
    content_encoding: String,
    content_type: String,
    target_path: path::PathBuf,
}

pub fn get_path_details_from_request(
    dir: &path::Path,
    req: &Request<IncomingBody>,
) -> (Option<(path::PathBuf, String)>, Option<String>) {
    // flatten path, convert to absolute (ie resolve ../../)
    let source_dir = match path::absolute(dir) {
        Ok(sdf) => sdf,
        _ => return (None, None),
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
        _ => return (None, None),
    };

    // confirm path resides in directory
    if !target_path.starts_with(source_dir) {
        // target path is 404
        return (None, None);
    }

    // get encoding details
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
        let encoded_path = match get_encoded_ext(enc) {
            // add_extension is a nightly feature on std::path
            // for now, get path as string, add ext, get path
            Some(ext) => {
                let updated_ext = path_lossy.to_string() + ext;
                path::PathBuf::from(updated_ext)
            }
            _ => continue,
        };

        if let Ok(exists) = encoded_path.try_exists() {
            if exists {
                return Some((encoded_path, enc.to_string()));
            }
        }
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
