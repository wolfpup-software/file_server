use hyper::body::Incoming as IncomingBody;
use hyper::service::Service;
use hyper::Request;
use std::future::Future;
use std::pin::Pin;

use responses;

use config::AvailableEncodings;
use config::ServiceRequirements;

pub struct Svc {
    service_requirements: ServiceRequirements,
    available_encodings: AvailableEncodings,
    ip_address: String,
}

impl Svc {
    pub fn new(
        service_requirements: &ServiceRequirements,
        available_encodings: &AvailableEncodings,
        ip_address: &str,
    ) -> Svc {
        Svc {
            service_requirements: service_requirements.clone(),
            available_encodings: available_encodings.clone(),
            ip_address: ip_address.to_string(),
        }
    }
}

impl Service<Request<IncomingBody>> for Svc {
    type Response = responses::BoxedResponse;
    type Error = hyper::http::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, req: Request<IncomingBody>) -> Self::Future {
        let path_details = responses::get_request_details(&req);

        Box::pin(async move { responses::build_response(&path_details) })
    }
}
