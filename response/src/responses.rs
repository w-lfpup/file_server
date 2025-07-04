use hyper::body::Incoming;
use hyper::http::Request;
use hyper::Method;
use hyper::StatusCode;

use crate::get_response::build_get_response;
use crate::head_response::build_head_response;
use crate::last_resort_response::build_last_resort_response;
use crate::type_flyweight::{BoxedResponse, ResponseParams};

pub const METHOD_NOT_ALLOWED_405: &str = "405 method not allowed";

pub async fn build_response(
    req: Request<Incoming>,
    res_params: ResponseParams,
) -> Result<BoxedResponse, hyper::http::Error> {
    match req.method() {
        &Method::GET => build_get_response(req, res_params).await,
        &Method::HEAD => build_head_response(req, res_params).await,
        _ => build_last_resort_response(StatusCode::METHOD_NOT_ALLOWED, METHOD_NOT_ALLOWED_405),
    }
}
