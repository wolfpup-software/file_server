use hyper::body::Incoming as IncomingBody;
use hyper::http::Request;
use hyper::service::Service;
use std::future::Future;
use std::pin::Pin;

use crate::responses;

use crate::config::Config;

pub struct Svc {
    pub config: Config,
}

impl Service<Request<IncomingBody>> for Svc {
    type Response = responses::BoxedResponse;
    type Error = hyper::http::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, req: Request<IncomingBody>) -> Self::Future {
        let paths = responses::get_paths_from_request(&self.config, &req);
        println!("{:?}", paths);
        let (content_type_and_target_path, encoding_type) =
            responses::get_path_details_from_request(&self.config.directory, &req);

        Box::pin(async {
            responses::build_response(content_type_and_target_path, encoding_type).await
        })
    }
}
