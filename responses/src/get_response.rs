use futures_util::TryStreamExt;
use http_body_util::{BodyExt, StreamBody};
use hyper::body::Frame;
use hyper::header::{CONTENT_ENCODING, CONTENT_LENGTH, CONTENT_TYPE};
use hyper::http::Response;
use tokio::fs::File;
use tokio_util::io::ReaderStream;

use crate::response_paths::PathDetails;
use crate::type_flyweight::BoxedResponse;

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

        // https://github.com/hyperium/hyper/blob/master/examples/send_file.rs
        let reader_stream = ReaderStream::new(file);
        let stream_body = StreamBody::new(reader_stream.map_ok(Frame::data));
        let boxed_body = stream_body.boxed();

        return Some(builder.body(boxed_body));
    }

    None
}
