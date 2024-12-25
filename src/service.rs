use hyper::body::Incoming as IncomingBody;
use hyper::http::Request;
use hyper::service::Service;
use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;

use crate::config::Config;
use crate::responses;

pub struct Svc {
    directory: PathBuf,
    available_encodings: responses::AvailableEncodings,
    filepath_404s: Vec<(PathBuf, Option<String>)>,
}

impl Svc {
    pub fn new(config: &Config, available_encodings: &responses::AvailableEncodings) -> Svc {
        Svc {
            directory: config.directory.clone(),
            available_encodings: available_encodings.clone(),
            filepath_404s: config.filepath_404s.clone(),
        }
    }
}

impl Service<Request<IncomingBody>> for Svc {
    type Response = responses::BoxedResponse;
    type Error = hyper::http::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, req: Request<IncomingBody>) -> Self::Future {
        // IF HEAD get details

        // ELSE any other request serves a file
        let paths =
            responses::get_paths_from_request(&self.directory, &self.available_encodings, &req);
        let filepath_404s = self.filepath_404s.clone();

        Box::pin(async move { responses::build_response_from_paths(filepath_404s, paths).await })
    }
}
