use hyper::body::Incoming as IncomingBody;
use hyper::http::Request;
use hyper::service::Service;
use std::future::Future;
use std::pin::Pin;

use crate::responses;

use crate::config::Config;

pub struct Svc {
    config: Config,
}

impl Svc {
    pub fn new(config: &Config) -> Svc {
        Svc {
            config: config.clone(),
        }
    }
}

impl Service<Request<IncomingBody>> for Svc {
    type Response = responses::BoxedResponse;
    type Error = hyper::http::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, req: Request<IncomingBody>) -> Self::Future {
        let paths = responses::get_paths_from_request(&self.config, &req);

        Box::pin(async move { responses::build_response_from_paths(paths).await })
    }
}
