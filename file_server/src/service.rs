use hyper::body::Incoming as IncomingBody;
use hyper::service::Service;
use hyper::Request;
use std::future::Future;
use std::pin::Pin;

use config::Config;
/*
    BoxedResponse is a type.
    It should work with hyper responses across
    different libraries and dependencies.
*/
use response::{build_response, BoxedResponse, ResponseParams};

#[derive(Clone, Debug)]
pub struct Svc {
    response_params: ResponseParams,
}

impl Svc {
    pub fn from(config: Config) -> Svc {
        Svc {
            response_params: ResponseParams::from(
                config.directory,
                config.filepath_404,
                config.content_encodings,
            ),
        }
    }
}

impl Service<Request<IncomingBody>> for Svc {
    type Response = BoxedResponse;
    type Error = hyper::http::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, req: Request<IncomingBody>) -> Self::Future {
        let response_params = self.response_params.clone();

        Box::pin(async move { build_response(req, response_params).await })
    }
}
