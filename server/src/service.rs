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
use responses::{BoxedResponse, ServiceRequirements};

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
        // cannot guarantee service_requirements isn't dropped
        let service_requirements = self.service_requirements.clone();

        Box::pin(async move { responses::build_response(req, service_requirements) })
    }
}
