use http_body_util::BodyExt;
use http_body_util::Full;
use hyper::header::{CONTENT_ENCODING, CONTENT_LENGTH, CONTENT_RANGE, CONTENT_TYPE};
use hyper::http::{Response, StatusCode};
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

// on any fail return nothing
fn get_ranges(range_str: &str, size: usize) -> Option<Vec<(usize, usize)>> {
    let stripped_range = range_str.trim();
    let range_values_str = match stripped_range.strip_prefix("bytes=") {
        Some(r) => r,
        _ => return None,
    };

    let mut ranges: Vec<(usize, usize)> = Vec::new();

    let split_values = range_values_str.split(",");
    for value_str in split_values {
        let trimmed_value_str = value_str.trim();
        // possible seek and read to end (N)
        if let Some(without_suffix) = trimmed_value_str.strip_suffix("-") {
            // parse int
            // push
            let start_range_int: usize = match without_suffix.parse() {
                Ok(sri) => sri,
                _ => return None,
            };

            ranges.push((start_range_int, size));
            continue;
        }

        // possible seek to (N - M) - N
        if let Some(without_prefix) = trimmed_value_str.strip_prefix("-") {
            // possible suffix
            let end_range_int: usize = match without_prefix.parse() {
                Ok(sri) => sri,
                _ => return None,
            };

            if end_range_int >= size {
                return None;
            }

            ranges.push((size - end_range_int, size));
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

fn get_content_range_header_str(start: &usize, end: &usize, size: &usize) -> String {
    "bytes ".to_string()
        + start.to_string().as_str()
        + "-"
        + end.to_string().as_str()
        + "/"
        + size.to_string().as_str()
}

pub async fn build_get_range_response_from_filepath(
    path_details: PathDetails,
    content_type: &str,
    range_str: &str,
) -> Option<Result<BoxedResponse, hyper::http::Error>> {
    let mut file_to_read = match File::open(path_details.path).await {
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

    // build response
    let mut builder = Response::builder()
        .status(StatusCode::PARTIAL_CONTENT)
        .header(CONTENT_TYPE, content_type);

    if let Some(enc) = path_details.content_encoding {
        builder = builder.header(CONTENT_ENCODING, enc);
    }

    // single range
    if 1 == ranges.len() {
        let (start, end) = match ranges.get(0) {
            Some(f) => f,
            _ => return None,
        };

        let _cursor = match file_to_read
            .seek(SeekFrom::Start(start.clone() as u64))
            .await
        {
            Ok(crsr) => crsr,
            _ => return None,
        };

        let mut buffer: Vec<u8> = Vec::with_capacity(end - start + 1);
        buffer.resize(end - start + 1, 0);

        if let Ok(_buffer_len) = file_to_read.read_exact(&mut buffer).await {
            let content_range_header = get_content_range_header_str(start, end, &size);
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
    }

    // multi
    if 1 < ranges.len() {

    }

    None
}
