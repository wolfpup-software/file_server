use http_body_util::{BodyExt, Full};
use hyper::body::Incoming as IncomingBody;
use hyper::header::{HeaderValue, ACCEPT_ENCODING, CONTENT_ENCODING, CONTENT_TYPE, RANGE};
use hyper::http::{Request, Response};
use hyper::StatusCode;
use std::path::PathBuf;

use crate::content_type::HTML;
// use crate::get_range_response::build_get_range_response_from_filepath;
// use crate::get_response::build_get_response_from_filepath;
// use crate::head_response::build_head_response_from_filepath;
// use crate::response_paths::ReqDetails;
use crate::type_flyweight::{BoxedResponse, RequestDetails};

pub const NOT_FOUND_416: &str = "416 requested range not satisfiable";
pub const NOT_FOUND_404: &str = "404 not found";

fn get_content_encodings_from_request(req: &Request<IncomingBody>) -> Option<Vec<String>> {
    let mut encodings = Vec::new();

    let accept_encoding_header = match req.headers().get(ACCEPT_ENCODING) {
        Some(enc) => enc,
        _ => return None,
    };

    let encoding_str = match accept_encoding_header.to_str() {
        Ok(s) => s,
        _ => return None,
    };

    for encoding in encoding_str.split(",") {
        encodings.push(encoding.trim().to_string());
    }

    Some(encodings)
}

fn get_range_from_request(req: &Request<IncomingBody>) -> Option<String> {
    if let Some(range_header) = req.headers().get(RANGE) {
        if let Ok(range_str) = range_header.to_str() {
            return Some(range_str.to_string());
        };
    };

    None
}

pub fn get_request_details(req: &Request<IncomingBody>) -> RequestDetails {
    RequestDetails {
        path: req.uri().path().to_string(),
        content_encoding: get_content_encodings_from_request(req),
        range: get_range_from_request(req),
    }
}
