use http_body_util::Full;
use http_body_util::{combinators::BoxBody, BodyExt};
use hyper::header::{CONTENT_ENCODING, CONTENT_LENGTH, CONTENT_TYPE};
use hyper::http::Response;
use tokio::fs::File;
use tokio::io;

use crate::response_paths::PathDetails;

pub type BoxedResponse = Response<BoxBody<bytes::Bytes, io::Error>>;

pub async fn build_get_response_from_filepath(
    path_details: PathDetails,
    content_type: &str,
) -> Option<Result<BoxedResponse, hyper::http::Error>> {
    if let Ok(file) = File::open(path_details.path).await {
        let metadata = match file.metadata().await {
            Ok(m) => m,
            _ => return None,
        };

        let mut builder = Response::builder()
            .status(path_details.status_code)
            .header(CONTENT_TYPE, content_type)
            .header(CONTENT_LENGTH, metadata.len());

        if let Some(enc) = path_details.content_encoding {
            builder = builder.header(CONTENT_ENCODING, enc);
        }

        return Some(
            builder.body(
                Full::new(bytes::Bytes::new())
                    .map_err(|e| match e {})
                    .boxed(),
            ),
        );
    }

    None
}
