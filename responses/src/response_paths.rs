use hyper::body::Incoming as IncomingBody;
use hyper::header::ACCEPT_ENCODING;
use hyper::http::Request;
use hyper::StatusCode;
use std::fs;
use std::path;
use std::path::{Path, PathBuf};

use crate::content_type::get_content_type;
use crate::type_flyweight::ServiceRequirements;

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
    pub path_details: Vec<(PathBuf, u64)>,
}

fn get_path_from_request_url(req: &Request<IncomingBody>) -> PathBuf {
    let uri_path = req.uri().path();

    let stripped = match uri_path.strip_prefix(FWD_SLASH) {
        Some(p) => p,
        _ => uri_path,
    };

    PathBuf::from(stripped)
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

pub fn get_filepaths_from_request(
    service_requirements: &ServiceRequirements,
    req: &Request<IncomingBody>,
) -> Vec<(PathBuf, Option<String>)> {
    let mut paths: Vec<(PathBuf, Option<String>)> = Vec::new();

    let req_path = get_path_from_request_url(req);

    // add fall_back urls
    push_fallback_paths(&mut paths, service_requirements);

    let encodings = get_encodings(req);

    return paths;
}

// let req_path = match get_path_from_request_url(&directory, req) {
//     Some(p) => p,
//     _ => return None,
// };

// let content_type = get_content_type(&req_path).to_string();
// let encodings = get_encodings(req);

// // try encoded paths first
// for encoding in encodings {
//     // if encoding not available skip
//     if !available_encodings.encoding_is_available(&encoding) {
//         continue;
//     }

//     let enc_from_ext = match get_encoded_ext(&encoding) {
//         Some(ext) => ext,
//         _ => continue,
//     };

//     let mut path_os_str = req_path.clone().into_os_string();
//     path_os_str.push(enc_from_ext);

//     let enc_path = path::PathBuf::from(path_os_str);

//     paths.push(PathDetails {
//         path: enc_path.clone(),
//         content_encoding: Some(encoding),
//         status_code: StatusCode::OK,
//     });
// }

// // push unencoded filepath
// paths.push(PathDetails {
//     path: req_path,
//     content_encoding: None,
//     status_code: StatusCode::OK,
// });

// // push 404s to file to serve
// for (filepath, encoding) in filepath_404s {
//     paths.push(PathDetails {
//         path: filepath.clone(),
//         content_encoding: encoding.clone(),
//         status_code: StatusCode::NOT_FOUND,
//     });
// }

// Some(ReqDetails {
//     content_type: content_type,
//     path_details: paths,
// })
