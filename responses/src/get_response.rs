use futures_util::TryStreamExt;
use http_body_util::{BodyExt, StreamBody};
use hyper::body::Frame;
use hyper::header::{CONTENT_ENCODING, CONTENT_LENGTH, CONTENT_TYPE};
use hyper::http::Response;
use hyper::StatusCode;
use std::path::Path;
use tokio::fs::File;
use tokio_util::io::ReaderStream;

use crate::type_flyweight::BoxedResponse;

pub async fn build_get_response_from_filepath(
    filepath: &Path,
    content_type: &str,
    status_code: StatusCode,
    content_encoding: &Option<String>,
) -> Option<Result<BoxedResponse, hyper::http::Error>> {
    let metadata = match tokio::fs::metadata(filepath).await {
        Ok(m) => m,
        _ => return None,
    };

    // if is_dir
    //
    // update path and directory

    if let Ok(file) = File::open(filepath).await {
        let mut builder = Response::builder()
            .status(status_code)
            .header(CONTENT_TYPE, content_type)
            .header(CONTENT_LENGTH, metadata.len());

        if let Some(enc) = content_encoding {
            builder = builder.header(CONTENT_ENCODING, enc);
        }

        // https://github.com/hyperium/hyper/blob/master/examples/send_file.rs
        let reader_stream = ReaderStream::new(file);
        let stream_body = StreamBody::new(reader_stream.map_ok(Frame::data));
        let boxed_body = stream_body.boxed();

        return Some(builder.body(boxed_body));
    }

    None
}
