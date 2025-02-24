use hyper::body::Incoming as IncomingBody;
use hyper::service::Service;
use hyper::Method;
use hyper::Request;
use hyper::StatusCode;
use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;

use responses;

use config::AvailableEncodings;
use config::ServiceRequirements;

pub struct Svc {
    service_requirements: ServiceRequirements,
    available_encodings: AvailableEncodings,
    ip_address: String,
}

// take a 
impl Svc {
    pub fn new(service_requirements: &ServiceRequirements, available_encodings: &AvailableEncodings, ip_address: &str) -> Svc {
        Svc {
            service_requirements: service_requirements.clone(),
            available_encodings:  available_encodings.clone(),
            ip_address: ip_address.to_string(),
        }
    }
}

impl Service<Request<IncomingBody>> for Svc {
    type Response = responses::BoxedResponse;
    type Error = hyper::http::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, req: Request<IncomingBody>) -> Self::Future {
        // get path details

        

        Box::pin(async move {
            responses::build_response()
        })
    }
}

// fn get_range_header_as_string(req: &Request<IncomingBody>) -> Option<String> {
//     if let Some(range_header) = req.headers().get("range") {
//         if let Ok(range_str) = range_header.to_str() {
//             return Some(range_str.to_string());
//         };
//     };

//     None
// }
