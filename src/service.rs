use hyper::body::Incoming as IncomingBody;
use hyper::http::Request;
use hyper::service::Service;
use std::future::Future;
use std::path;
use std::path::PathBuf;
use std::pin::Pin;

use crate::responses;

pub struct Svc {
    pub directory: path::PathBuf,
    pub filepath_404s: Vec<(PathBuf, Option<String>, Option<String>)>,
    pub filepath_500s: Vec<(PathBuf, Option<String>, Option<String>)>,
}

impl Service<Request<IncomingBody>> for Svc {
    type Response = responses::BoxedResponse;
    type Error = hyper::http::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, req: Request<IncomingBody>) -> Self::Future {
        let (content_type_and_target_path, encoding_type) =
            responses::get_path_details_from_request(&self.directory, &req);

        Box::pin(async {
            responses::build_response(content_type_and_target_path, encoding_type).await
        })
    }
}
