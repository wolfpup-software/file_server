use hyper::body::Incoming as IncomingBody;
use hyper::header::{ACCEPT_ENCODING, RANGE};
use hyper::http::Request;

use crate::type_flyweight::RequestDetails;

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
        method: req.method().clone(),
        path: req.uri().path().to_string(),
        content_encoding: get_content_encodings_from_request(req),
        range: get_range_from_request(req),
    }
}
