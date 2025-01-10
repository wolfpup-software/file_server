use http_body_util::{combinators::BoxBody, BodyExt, Full};
use hyper::header::{HeaderValue, CONTENT_TYPE};
use hyper::http::Response;
use hyper::StatusCode;
use tokio::io;

use crate::content_type::HTML;
use crate::get_response::build_get_response_from_filepath;
use crate::response_paths::ReqDetails;

pub const NOT_FOUND_404: &str = "404 not found";

pub type BoxedResponse = Response<BoxBody<bytes::Bytes, io::Error>>;

pub async fn build_get_response(
    opt_req_details: Option<ReqDetails>,
) -> Result<BoxedResponse, hyper::http::Error> {
    // this should include a 404 error response
    // happens with "directories" when no file is included
    if let Some(req_details) = opt_req_details {
        for path_detail in req_details.path_details {
            if let Some(res) =
                build_get_response_from_filepath(path_detail, &req_details.content_type).await
            {
                return res;
            }
        }
    };

    build_last_resort_response(StatusCode::NOT_FOUND, &NOT_FOUND_404)
}

pub fn build_last_resort_response(
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
