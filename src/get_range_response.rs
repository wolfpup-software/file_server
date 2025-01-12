use futures_util::TryStreamExt;
use http_body_util::{BodyExt, StreamBody};
use hyper::body::Frame;
use hyper::header::{CONTENT_ENCODING, CONTENT_LENGTH, CONTENT_TYPE};
use hyper::http::{Response, StatusCode};
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncSeekExt, SeekFrom};
use tokio_util::io::ReaderStream;

use crate::response_paths::PathDetails;
use crate::type_flyweight::BoxedResponse;

type FileChunk = tokio::io::Take<tokio::fs::File>;

// support single range request
// starts with bytes=
// then split with ","
// then split with "-"
//
// little logic to send single or multibyte response
//
// then parse int from split numbers
//

//

// Range: <unit>=<range-start>-
// Range: <unit>=<range-start>-<range-end>
// Range: <unit>=<range-start>-<range-end>, …, <range-startN>-<range-endN>
// Range: <unit>=-<suffix-length>

pub async fn build_get_range_response_from_filepath(
    path_details: PathDetails,
    content_type: &str,
    range_str: &str,
) -> Option<Result<BoxedResponse, hyper::http::Error>> {
    // get file

    // get file size

    // get range in bytes

    // get file size

    if let Ok(file) = File::open(path_details.path).await {
        let metadata = match file.metadata().await {
            Ok(m) => m,
            _ => return None,
        };

        let mut builder = Response::builder()
            .status(StatusCode::PARTIAL_CONTENT)
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

pub async fn read_file_range(filepath: &PathBuf) -> Option<FileChunk> {
    let start = 0;
    let length = 100;

    // let mut input = File::open("input.bin")?;

    // // Seek to the start position
    // input.seek(SeekFrom::Start(start))?;

    // // Create a reader with a fixed length
    // let mut chunk = input.take(length);

    // let mut output = File::create("output.bin")?;

    // // Copy the chunk into the output file
    // io::copy(&mut chunk, &mut output)?;

    let mut file_to_read = match File::open(filepath).await {
        Ok(ftr) => ftr,
        _ => return None,
    };

    let _ = file_to_read.seek(SeekFrom::Start(start));

    let chunk = file_to_read.take(length);

    Some(chunk)
}
