use bytes::Bytes;
use http_body_util::combinators::BoxBody;
use hyper::Response;
use std::path::{Component, PathBuf};
use tokio::io;

use crate::available_encodings::AvailableEncodings;

pub type BoxedResponse = Response<BoxBody<Bytes, io::Error>>;

pub const NOT_FOUND_404: &str = "404 not found";
pub const METHOD_NOT_ALLOWED_405: &str = "405 method not allowed";
pub const RANGE_NOT_SATISFIABLE_416: &str = "416 range not satisfiable";

#[derive(Clone, Debug)]
pub struct ResponseParams {
    pub directory: PathBuf,
    pub available_encodings: AvailableEncodings,
}

impl ResponseParams {
    pub fn from(directory: PathBuf, content_encodings: Option<Vec<String>>) -> ResponseParams {
        let available_encodings = AvailableEncodings::from(content_encodings);

        ResponseParams {
            directory,
            available_encodings,
        }
    }
}

pub fn normalize_uri_path_lexically(path_buf: &PathBuf) -> Option<PathBuf> {
    let mut parts: Vec<Component> = Vec::new();
    let mut debts: Vec<Component> = Vec::new();

    for component in path_buf.components() {
        match component {
            Component::ParentDir => {
                if let None = parts.pop() {
                    return None;
                };
            }
            Component::Normal(_) => {
                if let None = debts.pop() {
                    parts.push(component);
                }
            }
            _ => {}
        }
    }

    let mut normalized_uri_path = PathBuf::new();
    for component in parts {
        if let Component::Normal(os_str) = component {
            normalized_uri_path.push(os_str);
        }
    }

    Some(normalized_uri_path)
}
