use hyper::body::Incoming as IncomingBody;
use hyper::service::Service;
use hyper::Request;
use std::future::Future;
use std::pin::Pin;

/*
    BoxedResponse is a type.
    It should work with hyper responses across
    different libraries and dependencies.
*/

use responses::{AvailableEncodings, BoxedResponse, ServiceRequirements};

pub struct Svc {
    service_requirements: ServiceRequirements,
}

impl Svc {
    pub fn new(service_requirements: &ServiceRequirements) -> Svc {
        Svc {
            service_requirements: service_requirements.clone(),
        }
    }
}

impl Service<Request<IncomingBody>> for Svc {
    type Response = BoxedResponse;
    type Error = hyper::http::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, req: Request<IncomingBody>) -> Self::Future {
        let path_details = responses::get_request_details(&req);

        Box::pin(async move { responses::build_response(&path_details) })
    }
}
