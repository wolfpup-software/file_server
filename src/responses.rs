use std::path;
use std::path::{Path, PathBuf};

use futures_util::TryStreamExt;
use http_body_util::{combinators::BoxBody, BodyExt, Full, StreamBody};
use hyper::body::{Frame, Incoming as IncomingBody};
use hyper::header::{HeaderValue, ACCEPT_ENCODING, CONTENT_ENCODING, CONTENT_TYPE};
use hyper::http::{Request, Response};
use hyper::StatusCode;
use tokio::fs::File;
use tokio::io;
use tokio_util::io::ReaderStream;

use crate::content_encoding::{get_encoded_ext, AvailableEncodings};
use crate::content_type::{get_content_type, HTML};

const FWD_SLASH: &str = "/";
const INDEX: &str = "index.html";
const NOT_FOUND_404: &str = "404 not found";

pub type BoxedResponse = Response<BoxBody<bytes::Bytes, io::Error>>;

#[derive(Debug)]
pub struct PathDetails {
    pub path: PathBuf,
    pub status_code: StatusCode,
    pub content_encoding: Option<String>,
}

#[derive(Debug)]
pub struct ReqDetails {
    pub content_type: String,
    pub path_details: Vec<PathDetails>,
}

fn get_path_from_request_url(dir: &Path, req: &Request<IncomingBody>) -> Option<PathBuf> {
    let uri_path = req.uri().path();
    let mut target_path = match uri_path.strip_prefix(FWD_SLASH) {
        Some(p) => dir.join(p),
        _ => dir.join(uri_path),
    };

    // if directory add index.html
    if target_path.is_dir() {
        target_path.push(INDEX);
    }

    // confirm path resides in directory
    if target_path.starts_with(dir) {
        return Some(target_path);
    }

    None
}

fn get_encodings(req: &Request<IncomingBody>) -> Vec<String> {
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

pub fn get_filepaths_from_request(
    directory: &PathBuf,
    available_encodings: &AvailableEncodings,
    filepath_404s: &Vec<(PathBuf, Option<String>)>,
    req: &Request<IncomingBody>,
) -> Option<ReqDetails> {
    let mut paths = Vec::new();

    let req_path = match get_path_from_request_url(&directory, req) {
        Some(p) => p,
        _ => return None,
    };

    let content_type = get_content_type(&req_path).to_string();
    let encodings = get_encodings(req);
    println!("encodings: {:?} {:?}", available_encodings, encodings);

    // try encoded paths first
    for encoding in encodings {
        // if encoding not available skip
        if !available_encodings.encoding_is_available(&encoding) {
            continue;
        }

        let enc_from_ext = match get_encoded_ext(&encoding) {
            Some(ext) => ext,
            _ => continue,
        };

        let mut path_os_str = req_path.clone().into_os_string();
        path_os_str.push(enc_from_ext);

        let enc_path = path::PathBuf::from(path_os_str);

        paths.push(PathDetails {
            path: enc_path.clone(),
            content_encoding: Some(encoding),
            status_code: StatusCode::OK,
        });
    }

    // push unencoded filepath
    paths.push(PathDetails {
        path: req_path,
        content_encoding: None,
        status_code: StatusCode::OK,
    });

    // push 404s to file to serve
    for (filepath, encoding) in filepath_404s {
        paths.push(PathDetails {
            path: filepath.clone(),
            content_encoding: encoding.clone(),
            status_code: StatusCode::NOT_FOUND,
        });
    }

    Some(ReqDetails {
        content_type: content_type,
        path_details: paths,
    })
}

pub async fn build_response_from_filepaths(
    opt_req_details: Option<ReqDetails>,
) -> Result<BoxedResponse, hyper::http::Error> {
    // this should include a 404 error response
    // happens with "directories" when no file is included
    if let Some(req_details) = opt_req_details {
        for path_detail in req_details.path_details {
            if let Some(res) =
                create_response_from_filepath(path_detail, &req_details.content_type).await
            {
                return res;
            }
        }
    };

    create_not_found_response(&StatusCode::NOT_FOUND, &NOT_FOUND_404)
}

async fn create_response_from_filepath(
    path_details: PathDetails,
    content_type: &str,
) -> Option<Result<BoxedResponse, hyper::http::Error>> {
    if let Ok(file) = File::open(path_details.path).await {
        let mut builder = Response::builder()
            .status(path_details.status_code)
            .header(CONTENT_TYPE, content_type);

        if let Some(enc) = path_details.content_encoding {
            builder = builder.header(CONTENT_ENCODING, enc);
        }

        // https://github.com/hyperium/hyper/blob/master/examples/send_file.rs
        let reader_stream = ReaderStream::new(file);
        let stream_body = StreamBody::new(reader_stream.map_ok(Frame::data));
        let boxed_body = stream_body.boxed();

        return Some(builder.body(boxed_body));
    }

    None
}

pub fn create_not_found_response(
    status_code: &StatusCode,
    body: &'static str,
) -> Result<BoxedResponse, hyper::http::Error> {
    Response::builder()
        .status(status_code)
        .header(CONTENT_TYPE, HeaderValue::from_static(HTML))
        .body(
            Full::new(bytes::Bytes::from(body))
                .map_err(|e| match e {})
                .boxed(),
        )
}
