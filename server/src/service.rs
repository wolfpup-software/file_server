use hyper::body::Incoming as IncomingBody;
use hyper::service::Service;
use hyper::Request;
use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;

/*
    BoxedResponse is a type.
    It should work with hyper responses across
    different libraries and dependencies.
*/
use responses::BoxedResponse;

#[derive(Clone, Debug)]
pub struct Svc {
    directory: PathBuf,
    content_encodings: Option<Vec<String>>,
    fallback_404: Option<PathBuf>,
}

impl Svc {
    pub fn new(
        directory: PathBuf,
        content_encodings: Option<Vec<String>>,
        fallback_404: Option<PathBuf>,
    ) -> Svc {
        Svc {
            directory: directory,
            content_encodings: content_encodings,
            fallback_404: fallback_404,
        }
    }
}

impl Service<Request<IncomingBody>> for Svc {
    type Response = BoxedResponse;
    type Error = hyper::http::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, req: Request<IncomingBody>) -> Self::Future {
        let directory = self.directory.clone();
        let content_encodings = self.content_encodings.clone();
        let fallback_404 = self.fallback_404.clone();

        Box::pin(async move {
            responses::build_response(req, directory, content_encodings, fallback_404).await
        })
    }
}
