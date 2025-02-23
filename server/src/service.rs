use hyper::body::Incoming as IncomingBody;
use hyper::service::Service;
use hyper::Method;
use hyper::Request;
use hyper::StatusCode;
use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;

use config::content_encoding::AvailableEncodings;
use config::Config;

// use crate::response_paths::{get_filepaths_from_request, get_path_from_request};
// use crate::responses;
// use crate::type_flyweight::BoxedResponse;

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

// impl Service<Request<IncomingBody>> for Svc {
//     type Response = BoxedResponse;
//     type Error = hyper::http::Error;
//     type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

//     fn call(&self, req: Request<IncomingBody>) -> Self::Future {
//         // get potential filepaths
//         let path = get_path_from_request(&self.directory, &req);

//         let paths = get_filepaths_from_request(
//             &self.directory,
//             &self.available_encodings,
//             &self.filepath_404s,
//             &req,
//         );

//         // head request
//         if Method::HEAD == req.method() {
//             return Box::pin(async move { responses::build_head_response(paths).await });
//         }

//         if Method::GET == req.method() {
//             // range request
//             if let Some(range_header_string) = get_range_header_as_string(&req) {
//                 return Box::pin(async move {
//                     responses::build_get_range_response(paths, range_header_string).await
//                 });
//             };

//             // get request
//             return Box::pin(async move { responses::build_get_response(paths).await });
//         }

//         // not found
//         Box::pin(async move {
//             responses::build_last_resort_response(StatusCode::NOT_FOUND, responses::NOT_FOUND_404)
//         })
//     }
// }

// fn get_range_header_as_string(req: &Request<IncomingBody>) -> Option<String> {
//     if let Some(range_header) = req.headers().get("range") {
//         if let Ok(range_str) = range_header.to_str() {
//             return Some(range_str.to_string());
//         };
//     };

//     None
// }
