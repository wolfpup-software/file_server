use futures_util::TryStreamExt;
use http_body_util::{BodyExt, Full, StreamBody};
use hyper::body::Frame;
use hyper::header::{CONTENT_ENCODING, CONTENT_LENGTH, CONTENT_RANGE, CONTENT_TYPE};
use hyper::http::{Response, StatusCode};
use std::fs::Metadata;
use std::io::SeekFrom;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use tokio_util::io::ReaderStream;

use crate::response_paths::PathDetails;
use crate::type_flyweight::BoxedResponse;

// Range: <unit>=<range-start>-
// Range: <unit>=<range-start>-<range-end>
// Range: <unit>=-<suffix-length>

// multi range requests require an entire different strategy
// Range: <unit>=<range-start>-<range-end>, …, <range-startN>-<range-endN>

// on any fail return nothing
fn get_ranges(range_str: &str, size: usize) -> Option<Vec<(usize, usize)>> {
    let stripped_range = range_str.trim();
    let range_values_str = match stripped_range.strip_prefix("bytes=") {
        Some(r) => r,
        _ => return None,
    };

    // track the last
    let mut last_last = 0;

    let mut ranges: Vec<(usize, usize)> = Vec::new();
    for value_str in range_values_str.split(",") {
        let trimmed_value_str = value_str.trim();

        // Range: <unit>=-<suffix-length>
        //
        // M is the byte size of file
        // N is the start index
        // possible seek and read from N -> M
        //
        if let Some(without_suffix) = trimmed_value_str.strip_suffix("-") {
            // parse int
            // push
            let start_range_int: usize = match without_suffix.parse() {
                Ok(sri) => sri,
                _ => return None,
            };

            // overlap
            if start_range_int < last_last {
                return None;
            }
            last_last = size;

            ranges.push((start_range_int, size));
            continue;
        }

        // Range: <unit>=<range-start>-
        //
        // M is byte size of file
        // N is the
        // possible seek and read from (M - N) -> M
        //
        if let Some(without_prefix) = trimmed_value_str.strip_prefix("-") {
            // possible suffix
            let end_range_int: usize = match without_prefix.parse() {
                Ok(sri) => sri,
                _ => return None,
            };

            if end_range_int >= size {
                return None;
            }

            let start_range_int = size - end_range_int;

            // overlap
            if start_range_int < last_last {
                return None;
            }
            last_last = size;

            ranges.push((start_range_int, size));
            continue;
        }

        // Range: <unit>=<range-start>-<range-end>
        //
        let mut values = trimmed_value_str.split("-");

        let start_range_str = match values.next() {
            Some(start_range) => start_range,
            _ => return None,
        };

        let end_range_str = match values.next() {
            Some(end_range) => end_range,
            _ => return None,
        };

        let start_range_int: usize = match start_range_str.parse() {
            Ok(sri) => sri,
            _ => return None,
        };

        let end_range_int: usize = match end_range_str.parse() {
            Ok(sri) => sri,
            _ => return None,
        };

        // check bounds
        // overlap
        if start_range_int < last_last {
            return None;
        }
        last_last = end_range_int;

        if start_range_int > end_range_int {
            return None;
        }

        // swap for "size", never read more than size
        if end_range_int > size {
            return None;
        }

        ranges.push((start_range_int, end_range_int))
    }

    return Some(ranges);
}

fn build_content_range_header_str(start: &usize, end: &usize, size: &usize) -> String {
    "bytes ".to_string() + &start.to_string() + "-" + &end.to_string() + "/" + &size.to_string()
}

pub async fn build_get_range_response_from_filepath(
    path_details: PathDetails,
    content_type: &str,
    range_str: &str,
) -> Option<Result<BoxedResponse, hyper::http::Error>> {
    let file_to_read = match File::open(&path_details.path).await {
        Ok(f) => f,
        _ => return None,
    };

    let metadata = match file_to_read.metadata().await {
        Ok(m) => m,
        _ => return None,
    };

    let size = metadata.len() as usize;

    let ranges = match get_ranges(range_str, size) {
        Some(rngs) => rngs,
        _ => return None,
    };

    if 0 == ranges.len() {
        return None;
    }

    // single range
    if 1 == ranges.len() {
        // build response
        return build_single_range_response(
            file_to_read,
            metadata,
            path_details,
            ranges,
            content_type,
        )
        .await;
    }

    None
}

async fn build_single_range_response(
    mut file_to_read: File,
    metadata: Metadata,
    path_details: PathDetails,
    ranges: Vec<(usize, usize)>,
    content_type: &str,
) -> Option<Result<BoxedResponse, hyper::http::Error>> {
    // build response
    let size = metadata.len() as usize;

    let (start, end) = match ranges.get(0) {
        Some(f) => f,
        _ => return None,
    };

    let _cursor = match file_to_read
        // this could be problematic for other systems that are not u64
        .seek(SeekFrom::Start(start.clone() as u64))
        .await
    {
        Ok(crsr) => crsr,
        _ => return None,
    };

    let mut buffer: Vec<u8> = Vec::with_capacity(end - start + 1);
    buffer.resize(end - start + 1, 0);

    let content_range_header = build_content_range_header_str(start, end, &size);
    let reader_stream = ReaderStream::with_capacity(file_to_read, size);
    let stream_body = StreamBody::new(reader_stream.map_ok(Frame::data));
    let boxed_body = stream_body.boxed();

    let mut builder = Response::builder()
        .status(StatusCode::PARTIAL_CONTENT)
        .header(CONTENT_TYPE, content_type)
        .header(CONTENT_RANGE, content_range_header)
        .header(CONTENT_LENGTH, buffer.len().to_string());

    if let Some(enc) = path_details.content_encoding {
        builder = builder.header(CONTENT_ENCODING, enc);
    }

    return Some(builder.body(boxed_body));
}

// Multi-part ranges are particularly difficult to stream frame by frame.
// Solution could be a facade buffer that can:
//   - read a file frame by frame
//   - over multiple ranges
//   - while including boundaries
//
// Short solution is to limit file length.
//
// For now it seems not worth including.
//
async fn build_multipart_range_response(
    mut file_to_read: File,
    metadata: Metadata,
    path_details: PathDetails,
    ranges: Vec<(usize, usize)>,
    content_type: &str,
) -> Option<Result<BoxedResponse, hyper::http::Error>> {
    let boundary_core = "k1ng_0f_1nf1n1t3_5pac3";
    let boundary = "--".to_string() + boundary_core;

    // create a byte array
    let mut big_buffer: Vec<u8> = Vec::new();

    for range in ranges {
        let (start, end) = range;

        let _cursor = match file_to_read
            // this could be problematic for other systems that are not u64
            .seek(SeekFrom::Start(start.clone() as u64))
            .await
        {
            Ok(crsr) => crsr,
            _ => return None,
        };

        let mut buffer: Vec<u8> = Vec::with_capacity(end - start + 1);
        buffer.resize(end - start + 1, 0);

        // read file
        if let Err(_e) = file_to_read.read_exact(&mut buffer).await {
            return None;
        };

        // then add boundary
        big_buffer.extend(boundary.as_bytes());
        big_buffer.extend("\n".as_bytes());

        // then add headers
        let multi_headers = CONTENT_TYPE.to_string()
            + ": "
            + content_type
            + "\n"
            // gotta add content encoding
            + CONTENT_RANGE.as_str()
            + ": bytes "
            + &start.to_string()
            + "-"
            + &end.to_string()
            + "/"
            + &metadata.len().to_string()
            + "\n\n";
        big_buffer.extend(multi_headers.as_bytes());
        big_buffer.extend(buffer);
        big_buffer.extend("\n".as_bytes());
    }

    // add final boundary
    big_buffer.extend(boundary.as_bytes());
    big_buffer.extend("--".as_bytes());

    // Now build a response
    let req_content_type = "multipart/byteranges; boundary=".to_string() + boundary_core;

    let mut builder = Response::builder()
        .status(StatusCode::PARTIAL_CONTENT)
        .header(CONTENT_TYPE, req_content_type);

    if let Some(enc) = path_details.content_encoding {
        builder = builder.header(CONTENT_ENCODING, enc);
    }

    return Some(
        builder.header(CONTENT_LENGTH, big_buffer.len()).body(
            Full::new(bytes::Bytes::from(big_buffer))
                .map_err(|e| match e {})
                .boxed(),
        ),
    );
}
