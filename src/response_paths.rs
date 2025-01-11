use hyper::body::Incoming as IncomingBody;
use hyper::header::ACCEPT_ENCODING;
use hyper::http::Request;
use hyper::StatusCode;
use std::path;
use std::path::{Path, PathBuf};

use crate::content_encoding::{get_encoded_ext, AvailableEncodings};
use crate::content_type::get_content_type;

const FWD_SLASH: &str = "/";
const INDEX: &str = "index.html";

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

    // LOGIC:
    // if directory add index.html
    //
    // The path.is_dir() function checks if filepath exists on disk
    // causing `file_server` to serve a 404 with the incorrect
    // content_type: octet-media.
    //
    // Check if path is a file to give `file_server` a chance
    // to return a file _without an extension_ as an octet-media.
    //
    // This also helps return a 404 with the content_type `text/html`
    // if the directory does not exist
    if !target_path.is_file() {
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
