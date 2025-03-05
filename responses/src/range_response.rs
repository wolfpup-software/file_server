use futures_util::TryStreamExt;
use http_body_util::{BodyExt, StreamBody};
use hyper::body::Frame;
use hyper::body::Incoming as IncomingBody;
use hyper::header::{CONTENT_ENCODING, CONTENT_LENGTH, CONTENT_RANGE, CONTENT_TYPE, RANGE};
use hyper::http::{Request, Response, StatusCode};
use std::io::SeekFrom;
use std::path::PathBuf;
use tokio::fs;
use tokio::fs::File;
use tokio::io::AsyncSeekExt;
use tokio_util::io::ReaderStream;

use crate::content_type::get_content_type;
use crate::last_resort_response::{build_last_resort_response, NOT_FOUND_404};
use crate::response_paths::{add_extension, get_encodings, get_path_from_request_url};
use crate::type_flyweight::BoxedResponse;

// Range: <unit>=<range-start>-
// Range: <unit>=<range-start>-<range-end>
// Range: <unit>=-<suffix-length>

// multi range requests require an entirely different strategy
// Range: <unit>=<range-start>-<range-end>, …, <range-startN>-<range-endN>

pub const RANGE_NOT_SATISFIABLE_416: &str = "416 range not satisfiable";

pub async fn build_range_response(
    req: &Request<IncomingBody>,
    directory: &PathBuf,
    content_encodings: &Option<Vec<String>>,
) -> Option<Result<BoxedResponse, hyper::http::Error>> {
    let range_header = match get_range_header(req) {
        Some(rh) => rh,
        _ => return None,
    };

    let ranges = match get_ranges(&range_header) {
        Some(rngs) => rngs,
        _ => {
            return Some(build_last_resort_response(
                StatusCode::RANGE_NOT_SATISFIABLE,
                RANGE_NOT_SATISFIABLE_416,
            ))
        }
    };

    let filepath = match get_path_from_request_url(&req, &directory).await {
        Some(fp) => fp,
        _ => {
            return Some(build_last_resort_response(
                StatusCode::NOT_FOUND,
                NOT_FOUND_404,
            ))
        }
    };

    let content_type = get_content_type(&filepath);
    let encodings = get_encodings(&req, &content_encodings);

    // single range
    if 1 == ranges.len() {
        if let Some(res) = build_range_responses(
            &filepath,
            &content_type,
            StatusCode::PARTIAL_CONTENT,
            &encodings,
            ranges,
        )
        .await
        {
            return Some(res);
        }
    }

    Some(build_last_resort_response(
        StatusCode::NOT_FOUND,
        NOT_FOUND_404,
    ))
}

fn get_range_header(req: &Request<IncomingBody>) -> Option<String> {
    let accept_encoding_header = match req.headers().get(RANGE) {
        Some(enc) => enc,
        _ => return None,
    };

    match accept_encoding_header.to_str() {
        Ok(s) => Some(s.to_string()),
        _ => None,
    }
}

// on any fail return nothing
fn get_ranges(range_str: &str) -> Option<Vec<(Option<usize>, Option<usize>)>> {
    let stripped_range = range_str.trim();
    let range_values_str = match stripped_range.strip_prefix("bytes=") {
        Some(r) => r,
        _ => return None,
    };

    // track the last
    let mut last_last = Some(0);

    let mut ranges: Vec<(Option<usize>, Option<usize>)> = Vec::new();
    for value_str in range_values_str.split(",") {
        let trimmed_value_str = value_str.trim();

        // Range: <unit>=-<suffix-length>
        if let Some(without_suffix) = trimmed_value_str.strip_suffix("-") {
            let start_range_int: usize = match without_suffix.parse() {
                Ok(sri) => sri,
                _ => return None,
            };

            // if last_last None (end of file)

            ranges.push((Some(start_range_int), None));
            continue;
        }

        // Range: <unit>=<range-start>-
        if let Some(without_prefix) = trimmed_value_str.strip_prefix("-") {
            // possible suffix
            let end_range_int: usize = match without_prefix.parse() {
                Ok(sri) => sri,
                _ => return None,
            };

            ranges.push((None, Some(end_range_int)));
            continue;
        }

        // Range: <unit>=<range-start>-<range-end>
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
        if start_range_int > end_range_int {
            return None;
        }

        ranges.push((Some(start_range_int), Some(end_range_int)))
    }

    return Some(ranges);
}

fn build_content_range_header_str(start: &usize, end: &usize, size: &usize) -> String {
    "bytes ".to_string() + &start.to_string() + "-" + &end.to_string() + "/" + &size.to_string()
}

async fn build_range_responses(
    filepath: &PathBuf,
    content_type: &str,
    status_code: StatusCode,
    encodings: &Option<Vec<String>>,
    ranges: Vec<(Option<usize>, Option<usize>)>,
) -> Option<Result<BoxedResponse, hyper::http::Error>> {
    if let Some(res) =
        compose_enc_range_response(&filepath, content_type, status_code, &encodings, &ranges).await
    {
        return Some(res);
    };

    // origin target
    if let Some(res) =
        compose_single_range_response(&filepath, content_type, status_code, None, &ranges).await
    {
        return Some(res);
    }

    None
}

async fn compose_enc_range_response(
    filepath: &PathBuf,
    content_type: &str,
    status_code: StatusCode,
    encodings: &Option<Vec<String>>,
    ranges: &Vec<(Option<usize>, Option<usize>)>,
) -> Option<Result<BoxedResponse, hyper::http::Error>> {
    let encds = match encodings {
        Some(encds) => encds,
        _ => return None,
    };

    for enc in encds {
        if let Some(encoded_path) = add_extension(filepath, &enc) {
            if let Some(res) = compose_single_range_response(
                &encoded_path,
                content_type,
                status_code,
                Some(enc),
                ranges,
            )
            .await
            {
                return Some(res);
            }
        };
    }

    None
}

async fn compose_single_range_response(
    filepath: &PathBuf,
    content_type: &str,
    status_code: StatusCode,
    content_encoding: Option<&str>,
    ranges: &Vec<(Option<usize>, Option<usize>)>,
) -> Option<Result<BoxedResponse, hyper::http::Error>> {
    // build response

    // get size if file
    let metadata = match fs::metadata(filepath).await {
        Ok(m) => m,
        _ => return None,
    };

    if !metadata.is_file() {
        return None;
    }

    let size = metadata.len() as usize;
    let (start, end) = match ranges.get(0) {
        // suffix (M - N, M)
        Some((None, Some(end))) => (size - end, size),
        // prefix (N, M)
        Some((Some(start), None)) => (start.clone(), size),
        //
        Some((Some(start), Some(end))) => (start.clone(), end.clone()),
        _ => return None,
    };

    if end < start || size < start {
        return Some(build_last_resort_response(
            StatusCode::RANGE_NOT_SATISFIABLE,
            RANGE_NOT_SATISFIABLE_416,
        ));
    }

    let mut file = match File::open(filepath).await {
        Ok(m) => m,
        _ => return None,
    };

    if let Err(e) = file.seek(SeekFrom::Start(start.clone() as u64)).await {
        return None;
    };

    let mut buffer: Vec<u8> = Vec::with_capacity(end - start);
    buffer.resize(end - start, 0);

    let content_range_header = build_content_range_header_str(&start, &end, &size);
    let reader_stream = ReaderStream::with_capacity(file, size);
    let stream_body = StreamBody::new(reader_stream.map_ok(Frame::data));
    let boxed_body = stream_body.boxed();

    let mut builder = Response::builder()
        .status(status_code)
        .header(CONTENT_TYPE, content_type)
        .header(CONTENT_RANGE, content_range_header)
        .header(CONTENT_LENGTH, buffer.len().to_string());

    if let Some(enc) = content_encoding {
        builder = builder.header(CONTENT_ENCODING, enc);
    }

    return Some(builder.body(boxed_body));
}
