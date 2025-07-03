use hyper::body::Incoming;
use hyper::http::Request;
use hyper::Method;
use hyper::StatusCode;
use std::path::PathBuf;

use crate::available_encodings::AvailableEncodings;
use crate::get_response::build_get_response;
use crate::head_response::build_head_response;
use crate::last_resort_response::build_last_resort_response;
use crate::type_flyweight::BoxedResponse;

pub const METHOD_NOT_ALLOWED_405: &str = "405 method not allowed";

pub async fn build_response(
    req: Request<Incoming>,
    directory: PathBuf,
    available_encodings: AvailableEncodings,
    fallback_404: Option<PathBuf>,
) -> Result<BoxedResponse, hyper::http::Error> {
    match req.method() {
        &Method::HEAD => build_head_response(req, directory, available_encodings).await,
        &Method::GET => build_get_response(req, directory, available_encodings, fallback_404).await,
        _ => build_last_resort_response(StatusCode::METHOD_NOT_ALLOWED, METHOD_NOT_ALLOWED_405),
    }
}
