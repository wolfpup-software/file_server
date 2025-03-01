use hyper::body::Incoming as IncomingBody;
use hyper::header::ACCEPT_ENCODING;
use hyper::http::Request;
use std::path;
use std::path::{Path, PathBuf};
use tokio::fs;

use crate::available_encodings::{get_encoded_ext, AvailableEncodings};
use crate::content_type::get_content_type;
use crate::last_resort_response::build_last_resort_response;

pub async fn get_path_from_request_url(
    req: &Request<IncomingBody>,
    directory: &PathBuf,
) -> Option<PathBuf> {
    let uri_path = req.uri().path();

    let stripped = match uri_path.strip_prefix("/") {
        Some(p) => p,
        _ => uri_path,
    };

    let mut target_path = match path::absolute(directory.join(&stripped)) {
        Ok(pb) => pb,
        _ => return None,
    };

    // confirm path resides in directory
    if !target_path.starts_with(directory) {
        return None;
    }

    let mut mtdt = match fs::metadata(&target_path).await {
        Ok(sdf) => sdf,
        _ => return None,
    };

    // if file bail early
    if mtdt.is_file() {
        return Some(target_path);
    }

    if mtdt.is_dir() {
        target_path.push("index.html")
    }

    if let Ok(md) = fs::metadata(&target_path).await {
        return Some(target_path);
    }

    None
}

pub fn get_encodings(
    req: &Request<IncomingBody>,
    content_encodings: Option<Vec<String>>,
) -> Option<Vec<String>> {
    let accept_encoding_header = match req.headers().get(ACCEPT_ENCODING) {
        Some(enc) => enc,
        _ => return None,
    };

    let encoding_str = match accept_encoding_header.to_str() {
        Ok(s) => s,
        _ => return None,
    };

    let available_encodings = AvailableEncodings::new(content_encodings);
    let mut encodings = Vec::new();
    for encoding in encoding_str.split(",") {
        let trimmed = encoding.trim();
        if available_encodings.encoding_is_available(trimmed) {
            // get path with extension
            encodings.push(trimmed.to_string());
        }
    }

    encodings
}

fn get_target_path_from_path(dir: &Path, target_path: &str) -> Option<PathBuf> {
    let target_path_abs = match path::absolute(dir.join(&target_path)) {
        Ok(pb) => pb,
        _ => return None,
    };

    // confirm path resides in directory
    if target_path_abs.starts_with(dir) {
        return Some(target_path_abs);
    }

    None
}

fn push_encoded_paths(
    paths: &mut Vec<(PathBuf, Option<String>)>,
    req_path: &Path,
    encodings: &Vec<String>,
) {
    for encoding in encodings {
        if let Some(ext) = get_encoded_ext(encoding) {
            paths.push((req_path.join(ext), Some(ext.to_string())));
        }
    }
}
