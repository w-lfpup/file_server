use bytes::Bytes;
use http_body_util::combinators::BoxBody;
use hyper::Response;
use std::path::PathBuf;
use tokio::io;

use crate::available_encodings::AvailableEncodings;

pub type BoxedResponse = Response<BoxBody<Bytes, io::Error>>;

pub const NOT_FOUND_404: &str = "404 not found";
pub const BAD_REQUEST_400: &str = "400 bad request";

#[derive(Clone, Debug)]
pub struct ResponseParams {
    pub directory: PathBuf,
    pub available_encodings: AvailableEncodings,
    pub filepath_404: Option<PathBuf>,
}

impl ResponseParams {
    pub fn from(
        directory: PathBuf,
        filepath_404: Option<PathBuf>,
        content_encodings: Option<Vec<String>>,
    ) -> ResponseParams {
        let available_encodings = AvailableEncodings::from(content_encodings);

        ResponseParams {
            directory: directory,
            available_encodings: available_encodings,
            filepath_404: filepath_404,
        }
    }
}
