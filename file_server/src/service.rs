use crate::config::{Config, DirEntry};
use hyper::body::Incoming as IncomingBody;
use hyper::service::Service;
use hyper::Request;
use response::{compose_response, BoxedResponse, ResponseParams};
use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;

#[derive(Clone, Debug)]
pub struct Svc {
    url_paths: Vec<(DirEntry, ResponseParams)>,
}

impl Svc {
    pub fn from(config: &Config) -> Svc {
        Svc {
            url_paths: get_response_params(config),
        }
    }
}

impl Service<Request<IncomingBody>> for Svc {
    type Response = BoxedResponse;
    type Error = hyper::http::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, req: Request<IncomingBody>) -> Self::Future {
        let path = PathBuf::from(req.uri().path());
        println!("service called with: {:?}", &path);

        let mut params: Option<ResponseParams> = None;
        for (dir_entry, response_params) in &self.url_paths {
            if path.starts_with(&dir_entry.url_path_prefix) {
                params = Some(response_params.clone());
                break;
            }
        }

        println!("found request params: {:?}", params);
        Box::pin(async move { compose_response(req, params).await })
    }
}

fn get_response_params(config: &Config) -> Vec<(DirEntry, ResponseParams)> {
    let mut collection: Vec<(DirEntry, ResponseParams)> = Vec::new();
    for dir_entry in &config.directories {
        let tuple = (
            dir_entry.clone(),
            ResponseParams::from(&dir_entry.directory, &dir_entry.content_encodings),
        );

        collection.push(tuple);
    }

    collection
}
