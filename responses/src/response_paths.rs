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

fn get_path_from_request_url(req: &Request<IncomingBody>) -> PathBuf {
    let uri_path = req.uri().path();

    let stripped = match uri_path.strip_prefix(FWD_SLASH) {
        Some(p) => p,
        _ => uri_path,
    };

    PathBuf::from(stripped)
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

fn push_fallback_paths(
    paths: &mut Vec<(PathBuf, Option<String>)>,
    service_requirements: &ServiceRequirements,
) {
    if let Some(filepath_404s) = &service_requirements.filepath_404s {
        for (fallback_path, encoding) in filepath_404s {
            let mut encdng = None;
            if let Some(enc) = encoding {
                if service_requirements.encodings.encoding_is_available(enc) {
                    encdng = Some(enc.clone());
                }
            }

            let target_path =
                get_target_path_from_path(&service_requirements.directory, &fallback_path);
            if let Some(target_path) = target_path {
                paths.push((fallback_path.clone(), encdng));
            }
        }
    }
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

pub fn get_filepaths_from_request(
    service_requirements: &ServiceRequirements,
    req: &Request<IncomingBody>,
) -> Vec<(PathBuf, Option<String>)> {
    let mut paths: Vec<(PathBuf, Option<String>)> = Vec::new();

    // 404 fallbacks
    push_fallback_paths(&mut paths, service_requirements);

    // push source path
    let req_path = get_path_from_request_url(req);
    paths.push((req_path.clone(), None));

    let encodings = get_encodings(service_requirements, req);
    push_encoded_paths(&mut paths, &req_path, &encodings);

    return paths;
}
