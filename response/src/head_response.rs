use http_body_util::{BodyExt, Full};
use hyper::body::Incoming;
use hyper::header::{ACCEPT_RANGES, CONTENT_ENCODING, CONTENT_LENGTH, CONTENT_TYPE};
use hyper::http::{Request, Response};
use hyper::StatusCode;
use std::path::PathBuf;
use tokio::fs;

use crate::content_type::get_content_type;
use crate::last_resort_response;
use crate::response_paths::{add_extension, get_encodings, get_path_from_request_url};
use crate::type_flyweight::{BoxedResponse, ResponseParams, NOT_FOUND_404};

pub async fn build_response(
    req: Request<Incoming>,
    res_params: ResponseParams,
) -> Result<BoxedResponse, hyper::http::Error> {
    let encodings = get_encodings(&req, &res_params.available_encodings);

    if let Some(filepath) = get_path_from_request_url(&req, &res_params.directory).await {
        let content_type = get_content_type(&filepath);

        // encodings
        if let Some(res) = compose_encoded_response(&filepath, content_type, encodings).await {
            return res;
        };

        // origin target
        if let Some(res) = compose_response(&filepath, content_type, None).await {
            return res;
        }
    };

    last_resort_response::build_response(StatusCode::NOT_FOUND, NOT_FOUND_404)
}

async fn compose_encoded_response(
    filepath: &PathBuf,
    content_type: &str,
    content_encodings: Option<Vec<String>>,
) -> Option<Result<BoxedResponse, hyper::http::Error>> {
    let encodings = match content_encodings {
        Some(encds) => encds,
        _ => return None,
    };

    for content_encoding in encodings {
        if let Some(encoded_path) = add_extension(filepath, &content_encoding) {
            if let Some(res) =
                compose_response(&encoded_path, content_type, Some(content_encoding)).await
            {
                return Some(res);
            }
        };
    }

    None
}

async fn compose_response(
    filepath: &PathBuf,
    content_type: &str,
    content_encoding: Option<String>,
) -> Option<Result<BoxedResponse, hyper::http::Error>> {
    let metadata = match fs::metadata(filepath).await {
        Ok(m) => m,
        _ => return None,
    };

    if !metadata.is_file() {
        return None;
    }

    let mut builder = Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, content_type)
        .header(ACCEPT_RANGES, "bytes")
        .header(CONTENT_LENGTH, metadata.len());

    if let Some(enc) = content_encoding {
        builder = builder.header(CONTENT_ENCODING, enc);
    }

    Some(
        builder.body(
            Full::new(bytes::Bytes::new())
                .map_err(|e| match e {})
                .boxed(),
        ),
    )
}
