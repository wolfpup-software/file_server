use http_body_util::{BodyExt, Full};
use hyper::header::{ACCEPT_RANGES, CONTENT_ENCODING, CONTENT_LENGTH, CONTENT_TYPE};
use hyper::http::Response;
use hyper::StatusCode;
use std::path::Path;
use tokio::fs::File;

use crate::type_flyweight::BoxedResponse;

pub async fn build_head_response_from_filepath(
    filepath: &Path,
    content_type: &str,
    status_code: StatusCode,
    content_encoding: &Option<String>,
) -> Option<Result<BoxedResponse, hyper::http::Error>> {
    let file = match File::open(filepath).await {
        Ok(f) => f,
        _ => return None,
    };

    let metadata = match file.metadata().await {
        Ok(m) => m,
        _ => return None,
    };

    let mut builder = Response::builder()
        .status(status_code)
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
