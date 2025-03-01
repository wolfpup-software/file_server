use http_body_util::{BodyExt, Full};
use hyper::body::Incoming as IncomingBody;
use hyper::header::{ACCEPT_RANGES, CONTENT_ENCODING, CONTENT_LENGTH, CONTENT_TYPE};
use hyper::http::{Request, Response};
use hyper::StatusCode;
use std::path::Path;
use std::path::PathBuf;
use tokio::fs::File;

use crate::get_range_response::build_get_range_response;
use crate::last_resort_response::{build_last_resort_response, NOT_FOUND_404};
use crate::type_flyweight::BoxedResponse;

pub async fn build_head_response(
    req: Request<IncomingBody>,
    directory: PathBuf,
    content_encodings: Option<Vec<String>>,
) -> Result<BoxedResponse, hyper::http::Error> {
    // let file = match File::open(filepath).await {
    //     Ok(f) => f,
    //     _ => return None,
    // };

    // let metadata = match file.metadata().await {
    //     Ok(m) => m,
    //     _ => return None,
    // };

    // // is dir
    // // add index

    // let mut builder = Response::builder()
    //     .status(status_code)
    //     .header(CONTENT_TYPE, content_type)
    //     .header(ACCEPT_RANGES, "bytes")
    //     .header(CONTENT_LENGTH, metadata.len());

    // if let Some(enc) = content_encoding {
    //     builder = builder.header(CONTENT_ENCODING, enc);
    // }

    // Some(
    //     builder.body(
    //         Full::new(bytes::Bytes::new())
    //             .map_err(|e| match e {})
    //             .boxed(),
    //     ),
    // )

    build_last_resort_response(StatusCode::NOT_FOUND, NOT_FOUND_404)
}
