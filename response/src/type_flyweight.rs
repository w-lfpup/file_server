use bytes::Bytes;
use http_body_util::combinators::BoxBody;
use hyper::Response;
use std::path::PathBuf;
use tokio::io;

use crate::available_encodings::AvailableEncodings;

pub type BoxedResponse = Response<BoxBody<Bytes, io::Error>>;

#[derive(Clone, Debug)]
pub struct ResponseParams {
    pub directory: PathBuf,
    pub available_encodings: AvailableEncodings,
    pub filepath_404: Option<PathBuf>,
}
