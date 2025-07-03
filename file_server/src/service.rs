use hyper::body::Incoming as IncomingBody;
use hyper::service::Service;
use hyper::Request;
use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;

use crate::config::Config;
/*
    BoxedResponse is a type.
    It should work with hyper responses across
    different libraries and dependencies.
*/
use response::{build_response, AvailableEncodings, BoxedResponse};

#[derive(Clone, Debug)]
pub struct Svc {
    directory: PathBuf,
    available_encodings: AvailableEncodings,
    fallback_404: Option<PathBuf>,
}

impl Svc {
    pub fn from(config: Config) -> Svc {
        let available_encodings = AvailableEncodings::from(&config.content_encodings);

        Svc {
            directory: config.directory,
            available_encodings: available_encodings,
            fallback_404: config.filepath_404,
        }
    }
}

impl Service<Request<IncomingBody>> for Svc {
    type Response = BoxedResponse;
    type Error = hyper::http::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, req: Request<IncomingBody>) -> Self::Future {
        let directory = self.directory.clone();
        let available_encodings = self.available_encodings.clone();
        let fallback_404 = self.fallback_404.clone();

        Box::pin(
            async move { build_response(req, directory, available_encodings, fallback_404).await },
        )
    }
}
