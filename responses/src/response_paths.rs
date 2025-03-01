use hyper::body::Incoming as IncomingBody;
use hyper::header::ACCEPT_ENCODING;
use hyper::http::Request;
use std::path;
use std::path::{Path, PathBuf};

use crate::last_resort_response::build_last_resort_response;
use crate::type_flyweight::AvailableEncodings;

use crate::available_encodings::get_encoded_ext;
use crate::content_type::get_content_type;

fn get_path_from_request_url(req: &Request<IncomingBody>, directory: &PathBuf) -> Option<PathBuf> {
    let uri_path = req.uri().path();

    let stripped = match uri_path.strip_prefix("/") {
        Some(p) => p,
        _ => uri_path,
    };

    let target_path_abs = match path::absolute(directory.join(&stripped)) {
        Ok(pb) => pb,
        _ => return None,
    };

    // confirm path resides in directory
    if target_path_abs.starts_with(directory) {
        return Some(target_path_abs);
    }

    None
}

fn get_encodings(
    req: &Request<IncomingBody>,
    available_encodings: &AvailableEncodings,
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
        if available_encodings.encoding_is_available(trimmed) {
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

// pub fn get_filepaths_and_content_type_from_request(
//     req: &Request<IncomingBody>,
//     directory: &PathBuf,
//     available_encodings: &AvailableEncodings,
// ) -> (String, Vec<(PathBuf, Option<String>)>) {
//     let mut paths: Vec<(PathBuf, Option<String>)> = Vec::new();

//     // push source path
//     let req_path_str = get_path_from_request_url(req);

//     let target_path = get_target_path_from_path(directory, req_path_str);
//     let content_type = get_content_type(&target_path);
//     let encodings = get_encodings(req, available_encodings);

//     if let Some(tp) = &target_path {
//         push_encoded_paths(&mut paths, tp, &encodings);
//         paths.push((tp.clone(), None));
//     }

//     (content_type.to_string(), paths)
// }
