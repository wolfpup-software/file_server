use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;

use hyper::body::Incoming as IncomingBody;
use hyper::service::Service;
use hyper::Method;
use hyper::Request;
use hyper::StatusCode;

use crate::config::Config;
use crate::content_encoding::AvailableEncodings;
use crate::response_paths::get_filepaths_from_request;
use crate::responses::{
    build_get_response, build_last_resort_response, BoxedResponse, NOT_FOUND_404,
};

pub struct Svc {
    directory: PathBuf,
    available_encodings: AvailableEncodings,
    filepath_404s: Vec<(PathBuf, Option<String>)>,
}

impl Svc {
    pub fn new(config: &Config, available_encodings: &AvailableEncodings) -> Svc {
        Svc {
            directory: config.directory.clone(),
            available_encodings: available_encodings.clone(),
            filepath_404s: config.filepath_404s.clone(),
        }
    }
}

impl Service<Request<IncomingBody>> for Svc {
    type Response = BoxedResponse;
    type Error = hyper::http::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, req: Request<IncomingBody>) -> Self::Future {
        // IF HEAD get details

        // ELSE any other request serves a file
        let paths = get_filepaths_from_request(
            &self.directory,
            &self.available_encodings,
            &self.filepath_404s,
            &req,
        );

        if req.method() == Method::HEAD {}

        if req.method() == Method::GET {
            return Box::pin(async move { build_get_response(paths).await });
        }

        Box::pin(async move { build_last_resort_response(StatusCode::NOT_FOUND, NOT_FOUND_404) })
    }
}
