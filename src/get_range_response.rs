use http_body_util::BodyExt;
use http_body_util::Full;
use hyper::header::{CONTENT_ENCODING, CONTENT_LENGTH, CONTENT_RANGE, CONTENT_TYPE};
use hyper::http::{Response, StatusCode};
use std::fs::Metadata;
use std::io::SeekFrom;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncSeekExt};

use crate::response_paths::PathDetails;
use crate::type_flyweight::BoxedResponse;

// Range: <unit>=<range-start>-
// Range: <unit>=<range-start>-<range-end>
// Range: <unit>=-<suffix-length>

// multi range requests require an entire different strategy
// Range: <unit>=<range-start>-<range-end>, …, <range-startN>-<range-endN>

// what about a starts with ends with situation?

// if start range int is less than last last
// return None

//
//

// on any fail return nothing
fn get_ranges(range_str: &str, size: usize) -> Option<Vec<(usize, usize)>> {
    let stripped_range = range_str.trim();
    let range_values_str = match stripped_range.strip_prefix("bytes=") {
        Some(r) => r,
        _ => return None,
    };

    let mut ranges: Vec<(usize, usize)> = Vec::new();

    // keep reference to previous range
    // if next is out of order, return

    // track the last
    let mut last_last = 0;

    let split_values = range_values_str.split(",");
    for value_str in split_values {
        let trimmed_value_str = value_str.trim();
        // M is the byte size of file
        // N is the start index
        // possible seek and read from N -> M
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

        // M is byte size of file
        // N is the
        // possible seek and read from (M - N) -> M
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

        // start-end value range
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

    // multi
    if 1 < ranges.len() {
        // HTTP/1.1 206 Partial Content
        // Content-Type: multipart/byteranges; boundary=3d6b6a416f9b5
        // Content-Length: N
        //
        // --3d6b6a416f9b5
        // Content-Type: text/html
        // Content-Range: bytes 0-50/1270

        // <!doctype html>
        // <html lang="en-US">
        // <head>
        //     <title>Example Do
        // --3d6b6a416f9b5
        // Content-Type: text/html
        // Content-Range: bytes 100-150/1270

        // eta http-equiv="Content-type" content="text/html; c
        // --3d6b6a416f9b5--

        return build_multipart_range_response(
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

    let mut builder = Response::builder()
        .status(StatusCode::PARTIAL_CONTENT)
        .header(CONTENT_TYPE, content_type);

    if let Some(enc) = path_details.content_encoding {
        builder = builder.header(CONTENT_ENCODING, enc);
    }

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

    if let Ok(_buffer_len) = file_to_read.read_exact(&mut buffer).await {
        let content_range_header = build_content_range_header_str(start, end, &size);
        return Some(
            builder
                .header(CONTENT_LENGTH, buffer.len())
                .header(CONTENT_RANGE, content_range_header)
                .body(
                    Full::new(bytes::Bytes::from(buffer))
                        .map_err(|e| match e {})
                        .boxed(),
                ),
        );
    };

    None
}

async fn build_multipart_range_response(
    mut file_to_read: File,
    metadata: Metadata,
    path_details: PathDetails,
    ranges: Vec<(usize, usize)>,
    content_type: &str,
) -> Option<Result<BoxedResponse, hyper::http::Error>> {
    let boundary = "--delays_have_dangerous_ends\n";

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

        // then add
        big_buffer.extend(boundary.as_bytes());
        // "
        // Content-Type: text/html
        // Content-Range: bytes 0-50/1270
        // "
        let multi_headers = CONTENT_TYPE.to_string()
            + ": "
            + content_type
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
    }
    big_buffer.extend(boundary.as_bytes());
    big_buffer.extend("--".as_bytes());

    // Now build a response
    let content_length = big_buffer.len();

    // return a response
    None
}
