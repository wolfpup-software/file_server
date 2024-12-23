use hyper::body::Incoming as IncomingBody;
use hyper::http::Request;
use hyper::service::Service;
use std::future::Future;
use std::pin::Pin;

use crate::config::Config;
use crate::responses;

pub struct Svc {
    config: Config,
    av_enc: responses::AvailableEncodings,
}

impl Svc {
    pub fn new(config: &Config, av_enc: &responses::AvailableEncodings) -> Svc {
        Svc {
            config: config.clone(),
            av_enc: av_enc.clone(),
        }
    }
}

impl Service<Request<IncomingBody>> for Svc {
    type Response = responses::BoxedResponse;
    type Error = hyper::http::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, req: Request<IncomingBody>) -> Self::Future {
        let paths = responses::get_paths_from_request(&self.config, &req);
        let filepath_404s = self.config.filepath_404s.clone();

        Box::pin(async move { responses::build_response_from_paths(filepath_404s, paths).await })
    }
}
