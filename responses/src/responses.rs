use http_body_util::{BodyExt, Full};
use hyper::header::{HeaderValue, CONTENT_TYPE, CONTENT_ENCODING};
use hyper::http::{Request, Response};
use hyper::StatusCode;
use hyper::body::Incoming as IncomingBody;
use std::path::PathBuf;

use crate::content_type::HTML;
// use crate::get_range_response::build_get_range_response_from_filepath;
// use crate::get_response::build_get_response_from_filepath;
// use crate::head_response::build_head_response_from_filepath;
// use crate::response_paths::ReqDetails;
use crate::type_flyweight::BoxedResponse;

pub const NOT_FOUND_416: &str = "416 requested range not satisfiable";
pub const NOT_FOUND_404: &str = "404 not found";

#[derive(Debug)]
pub struct RequestDetails {
    pub path: String,
    pub content_encoding: Option<String>,
}

// pub async fn build_head_response(
//     opt_req_details: Option<ReqDetails>,
// ) -> Result<BoxedResponse, hyper::http::Error> {
//     // this should include a 404 error response
//     // happens with "directories" when no file is included
//     if let Some(req_details) = opt_req_details {
//         for path_detail in req_details.path_details {
//             if let Some(res) =
//                 build_head_response_from_filepath(path_detail, &req_details.content_type).await
//             {
//                 return res;
//             }
//         }
//     };

//     build_last_resort_response(StatusCode::NOT_FOUND, &NOT_FOUND_404)
// }

// pub async fn build_get_response(
//     opt_req_details: Option<ReqDetails>,
// ) -> Result<BoxedResponse, hyper::http::Error> {
//     // this should include a 404 error response
//     // happens with "directories" when no file is included
//     if let Some(req_details) = opt_req_details {
//         for path_detail in req_details.path_details {
//             if let Some(res) =
//                 build_get_response_from_filepath(path_detail, &req_details.content_type).await
//             {
//                 return res;
//             }
//         }
//     };

//     build_last_resort_response(StatusCode::NOT_FOUND, &NOT_FOUND_404)
// }

// pub async fn build_get_range_response(
//     opt_req_details: Option<ReqDetails>,
//     range_string: String,
// ) -> Result<BoxedResponse, hyper::http::Error> {
//     if let Some(req_details) = opt_req_details {
//         for path_detail in req_details.path_details {
//             if let Some(res) = build_get_range_response_from_filepath(
//                 path_detail,
//                 &req_details.content_type,
//                 &range_string,
//             )
//             .await
//             {
//                 return res;
//             }
//         }
//     };

//     build_last_resort_response(StatusCode::RANGE_NOT_SATISFIABLE, &NOT_FOUND_416)
// }

pub fn get_request_details(req: &Request<IncomingBody>) -> RequestDetails {
    let path = req.uri().path();

    let content_encoding = req.headers().get(CONTENT_ENCODING);
    let content_encoding_str = match content_encoding {
        Some(ce) => {
            match ce.to_str() {
                Ok(ce_str) => Some(ce_str.to_string()),
                _ => None,
            }
        },
        _ => None,
    };

    RequestDetails {
        path: path.to_string(),
        content_encoding: content_encoding_str,
    }
}

pub fn build_response() -> Result<BoxedResponse, hyper::http::Error> {
    build_last_resort_response(StatusCode::NOT_FOUND, NOT_FOUND_404)
}

fn build_last_resort_response(
    status_code: StatusCode,
    body: &'static str,
) -> Result<BoxedResponse, hyper::http::Error> {
    Response::builder()
        .status(status_code)
        .header(CONTENT_TYPE, HeaderValue::from_static(HTML))
        .body(
            Full::new(bytes::Bytes::from(body))
                .map_err(|e| match e {})
                .boxed(),
        )
}
