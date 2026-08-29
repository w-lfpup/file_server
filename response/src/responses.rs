use hyper::body::Incoming;
use hyper::http::Request;
use hyper::Method;
use hyper::StatusCode;

use crate::get_response;
use crate::head_response;
use crate::last_resort_response;
use crate::utils_flyweight::{
    BoxedResponse, ResponseParams, METHOD_NOT_ALLOWED_405, NOT_FOUND_404,
};

pub async fn compose_response(
    req: Request<Incoming>,
    res_params: Option<ResponseParams>,
) -> Result<BoxedResponse, hyper::http::Error> {
    let params = match res_params {
        Some(prms) => prms,
        _ => return last_resort_response::build_response(StatusCode::NOT_FOUND, NOT_FOUND_404),
    };

    match req.method() {
        &Method::GET => get_response::build_response(req, params).await,
        &Method::HEAD => head_response::build_response(req, params).await,
        _ => last_resort_response::build_response(
            StatusCode::METHOD_NOT_ALLOWED,
            METHOD_NOT_ALLOWED_405,
        ),
    }
}
