use http_body_util::{BodyExt, Full};
use hyper::body::Incoming as IncomingBody;
use hyper::header::{ACCEPT_RANGES, CONTENT_ENCODING, CONTENT_LENGTH, CONTENT_TYPE};
use hyper::http::{Request, Response};
use hyper::StatusCode;
use std::path::Path;
use std::path::PathBuf;
use tokio::fs;
use tokio::fs::File;

use crate::content_type::get_content_type;
use crate::get_range_response::build_get_range_response;
use crate::last_resort_response::{build_last_resort_response, NOT_FOUND_404};
use crate::response_paths::{get_encodings, get_path_from_request_url};
use crate::type_flyweight::BoxedResponse;

pub async fn build_head_response(
    req: Request<IncomingBody>,
    directory: PathBuf,
    content_encodings: Option<Vec<String>>,
) -> Result<BoxedResponse, hyper::http::Error> {
    let filepath = match get_path_from_request_url(&req, &directory).await {
        Some(fp) => fp,
        _ => return build_last_resort_response(StatusCode::NOT_FOUND, NOT_FOUND_404),
    };

    let content_type = get_content_type(&filepath);

    // let encodings
    if let Some(encodings) = get_encodings(&req, content_encodings) {
        // try and read encoding files
    };

    // origin target
    if let Some(res) = compose_head_response(&filepath, content_type, None).await {
        return res;
    }

    build_last_resort_response(StatusCode::NOT_FOUND, NOT_FOUND_404)
}

async fn compose_head_response(
    filepath: &PathBuf,
    content_type: &str,
    content_encoding: Option<String>,
) -> Option<Result<BoxedResponse, hyper::http::Error>> {
    let metadata = match fs::metadata(filepath).await {
        Ok(m) => m,
        _ => return None,
    };

    let mut builder = Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, content_type)
        .header(ACCEPT_RANGES, "bytes")
        .header(CONTENT_LENGTH, metadata.len());

    if let Some(enc) = content_encoding {
        builder = builder.header(CONTENT_ENCODING, enc);
    }

    Some(
        builder.body(
            Full::new(bytes::Bytes::new())
                .map_err(|e| match e {})
                .boxed(),
        ),
    )
}
