use http_body_util::{BodyExt, Full};
use hyper::body::Incoming as IncomingBody;
use hyper::header::{HeaderValue, ACCEPT_ENCODING, CONTENT_ENCODING, CONTENT_TYPE, RANGE};
use hyper::http::{Request, Response};
use hyper::Method;
use hyper::StatusCode;
use std::path::PathBuf;

use crate::content_type::HTML;
// use crate::get_range_response::build_get_range_response_from_filepath;
use crate::get_response::build_get_response_from_filepath;
use crate::head_response::build_head_response_from_filepath;
use crate::response_paths::get_filepaths_and_content_type_from_request;
use crate::type_flyweight::{BoxedResponse, ServiceRequirements};

pub const NOT_FOUND_416: &str = "416 requested range not satisfiable";
pub const NOT_FOUND_404: &str = "404 not found";
pub const METHOD_NOT_ALLOWED_405: &str = "405 method not allowed";

pub async fn build_response(
    req: Request<IncomingBody>,
    service_requirements: ServiceRequirements,
) -> Result<BoxedResponse, hyper::http::Error> {
    match req.method() {
        &Method::HEAD => build_head_response(req, service_requirements).await,
        &Method::GET => build_get_response(req, service_requirements).await,
        _ => build_last_resort_response(StatusCode::METHOD_NOT_ALLOWED, METHOD_NOT_ALLOWED_405),
    }
}

async fn build_head_response(
    req: Request<IncomingBody>,
    service_requirements: ServiceRequirements,
) -> Result<BoxedResponse, hyper::http::Error> {
    let (content_type, filepaths) =
        get_filepaths_and_content_type_from_request(&service_requirements, &req);

    for (filepath, encoding) in filepaths {
        if let Some(res) =
            build_head_response_from_filepath(&filepath, &content_type, StatusCode::OK, &encoding)
                .await
        {
            return res;
        }
    }

    build_last_resort_response(StatusCode::NOT_FOUND, NOT_FOUND_404)
}

async fn build_get_response(
    req: Request<IncomingBody>,
    service_requirements: ServiceRequirements,
) -> Result<BoxedResponse, hyper::http::Error> {
    let (content_type, filepaths) =
        get_filepaths_and_content_type_from_request(&service_requirements, &req);

    // see if it's a range request

    for (filepath, encoding) in filepaths {
        if let Some(res) =
            build_get_response_from_filepath(&filepath, &content_type, StatusCode::OK, &encoding)
                .await
        {
            return res;
        }
    }

    build_last_resort_response(StatusCode::NOT_FOUND, NOT_FOUND_404)
}

async fn build_range_response(
    req: Request<IncomingBody>,
    service_requirements: ServiceRequirements,
) -> Result<BoxedResponse, hyper::http::Error> {
    build_last_resort_response(StatusCode::RANGE_NOT_SATISFIABLE, NOT_FOUND_404)
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
