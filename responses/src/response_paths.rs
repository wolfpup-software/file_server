use hyper::body::Incoming as IncomingBody;
use hyper::header::ACCEPT_ENCODING;
use hyper::http::Request;
use hyper::StatusCode;
use std::fs;
use std::path;
use std::path::{Path, PathBuf};

use crate::available_encodings::get_encoded_ext;
use crate::content_type::get_content_type;
use crate::type_flyweight::ServiceRequirements;

const FWD_SLASH: &str = "/";
const INDEX: &str = "index.html";

fn get_path_from_request_url(req: &Request<IncomingBody>, directory: &PathBuf) -> PathBuf {
    let uri_path = req.uri().path();

    let stripped = match uri_path.strip_prefix(FWD_SLASH) {
        Some(p) => p,
        _ => uri_path,
    };

    directory.join(stripped)
}

fn get_encodings(
    service_requirements: &ServiceRequirements,
    req: &Request<IncomingBody>,
) -> Vec<String> {
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
        let trimmed = encoding.trim();
        if service_requirements
            .encodings
            .encoding_is_available(trimmed)
        {
            encodings.push(trimmed.to_string());
        }
    }

    encodings
}

fn get_target_path_from_path(dir: &Path, target_path: &Path) -> Option<PathBuf> {
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

pub fn get_filepaths_and_content_type_from_request(
    service_requirements: &ServiceRequirements,
    req: &Request<IncomingBody>,
) -> (String, Vec<(PathBuf, Option<String>)>) {
    let mut paths: Vec<(PathBuf, Option<String>)> = Vec::new();

    // push source path
    let req_path = get_path_from_request_url(req, &service_requirements.directory);
    let content_type = get_content_type(&req_path);
    paths.push((req_path.clone(), None));

    let encodings = get_encodings(service_requirements, req);
    push_encoded_paths(&mut paths, &req_path, &encodings);

    (content_type.to_string(), paths)
}
