use futures_util::TryStreamExt;
use http_body_util::{combinators::BoxBody, BodyExt, Full, StreamBody};
use hyper::body::{Frame, Incoming as IncomingBody};
use hyper::header::{HeaderValue, ACCEPT_ENCODING, CONTENT_ENCODING, CONTENT_TYPE};
use hyper::http::{Request, Response};
use hyper::StatusCode;
use std::path::{Path, PathBuf};
use std::{io, path};
use tokio::fs::File;
use tokio_util::io::ReaderStream;

use crate::content_and_encoding::{get_content_type, get_encoded_ext, HTML};

const FWD_SLASH: &str = "/";
const INDEX: &str = "index.html";
const INTERNAL_SERVER_ERROR: &str = "500 internal server error";
// const HTML: &str = "text/html; charset=utf-8";

// Preflight Checklist
//  Encodings
//  404s
//  Directory

#[derive(Clone, Debug)]
pub struct AvailableEncodings {
    gzip: bool,
    deflate: bool,
    br: bool,
    zstd: bool,
}

impl AvailableEncodings {
    pub fn new(potential_encodings: &Vec<String>) -> AvailableEncodings {
        let mut av_enc = AvailableEncodings {
            gzip: false,
            deflate: false,
            br: false,
            zstd: false,
        };

        for encoding in potential_encodings {
            match encoding.as_str() {
                "gzip" => av_enc.gzip = true,
                "deflate" => av_enc.deflate = true,
                "br" => av_enc.br = true,
                "zstd" => av_enc.zstd = true,
                _ => {}
            }
        }

        av_enc
    }
}

pub type BoxedResponse = Response<BoxBody<bytes::Bytes, io::Error>>;

#[derive(Debug)]
pub struct PathDetails {
    pub path: PathBuf,
    pub encoding: String,
}

#[derive(Debug)]
pub struct ReqDetails {
    pub path: PathBuf,
    pub content_type: String,
    pub path_details: Vec<PathDetails>,
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

pub fn get_paths_from_request(directory: &PathBuf, req: &Request<IncomingBody>) -> Option<ReqDetails> {
    let mut paths = Vec::new();

    let req_path = match get_path_from_request_url(&directory, req) {
        Some(p) => p,
        _ => return None,
    };

    let content_type = get_content_type(&req_path).to_string();
    let encodings = get_encodings(req);
    println!("{:?}", &encodings);

    for encoding in encodings {
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

pub async fn build_response_from_paths(
    filepath_404s: Vec<(PathBuf, Option<String>)>,
    opt_req_details: Option<ReqDetails>,
) -> Result<BoxedResponse, hyper::http::Error> {
    let req_details = match opt_req_details {
        Some(rd) => rd,
        _ => return serve_404s(filepath_404s).await,
    };

    // try encodings
    for path_detail in req_details.path_details {
        if let Some(res) = try_to_serve_filepath(
            path_detail.path,
            &req_details.content_type,
            Some(path_detail.encoding),
        )
        .await
        {
            return res;
        }
    }

    // try non encoded
    if let Some(res) =
        try_to_serve_filepath(req_details.path, &req_details.content_type, None).await
    {
        return res;
    }

    serve_404s(filepath_404s).await
}

async fn serve_404s(
    filepath_404s: Vec<(PathBuf, Option<String>)>,
) -> Result<BoxedResponse, hyper::http::Error> {
    // 404s just happens
    for (filepath, enc_type) in filepath_404s {
        if let Some(res) = try_to_serve_filepath(filepath, HTML, enc_type).await {
            return res;
        }
    }

    // finally default file not found
    create_error_response(&StatusCode::NOT_FOUND, &INTERNAL_SERVER_ERROR)
}

async fn try_to_serve_filepath(
    req_path: PathBuf,
    content_type: &str,
    enc_type: Option<String>,
) -> Option<Result<BoxedResponse, hyper::http::Error>> {
    if let Ok(file) = File::open(req_path).await {
        let mut builder = Response::builder()
            .status(StatusCode::OK)
            .header(CONTENT_TYPE, content_type);

        if let Some(enc) = enc_type {
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
