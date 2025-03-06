use futures_util::TryStreamExt;
use http_body_util::{BodyExt, StreamBody};
use hyper::body::{Frame, Incoming};
use hyper::header::{CONTENT_ENCODING, CONTENT_LENGTH, CONTENT_TYPE};
use hyper::http::{Request, Response};
use hyper::StatusCode;
use std::path;
use std::path::PathBuf;
use tokio::fs;
use tokio::fs::File;
use tokio_util::io::ReaderStream;

use crate::content_type::get_content_type;
use crate::last_resort_response::build_last_resort_response;
use crate::range_response::build_range_response;
use crate::response_paths::{add_extension, get_encodings, get_path_from_request_url};
use crate::type_flyweight::BoxedResponse;

pub const NOT_FOUND_404: &str = "404 not found";

pub async fn build_get_response(
    req: Request<Incoming>,
    directory: PathBuf,
    content_encodings: Option<Vec<String>>,
    fallback_404: Option<PathBuf>,
) -> Result<BoxedResponse, hyper::http::Error> {
    // check for range request
    if let Some(res) = build_range_response(&req, &directory, &content_encodings).await {
        return res;
    }

    // request file
    let encodings = get_encodings(&req, &content_encodings);

    // serve file
    if let Some(res) = build_file_response(&req, &directory, &encodings).await {
        return res;
    };

    // serve 404
    if let Some(res) = build_not_found_response(&directory, &fallback_404, &encodings).await {
        return res;
    };

    build_last_resort_response(StatusCode::NOT_FOUND, NOT_FOUND_404)
}

async fn build_file_response(
    req: &Request<Incoming>,
    directory: &PathBuf,
    encodings: &Option<Vec<String>>,
) -> Option<Result<BoxedResponse, hyper::http::Error>> {
    let filepath = match get_path_from_request_url(req, directory).await {
        Some(fp) => fp,
        _ => return None,
    };

    build_response(&filepath, StatusCode::OK, &encodings).await
}

async fn build_not_found_response(
    directory: &PathBuf,
    fallback_404: &Option<PathBuf>,
    encodings: &Option<Vec<String>>,
) -> Option<Result<BoxedResponse, hyper::http::Error>> {
    let fallback = match fallback_404 {
        Some(fb) => fb,
        _ => return None,
    };

    // file starts with directory
    let fallback_404_abs = match path::absolute(fallback) {
        Ok(fb) => fb,
        _ => return None,
    };

    if !fallback_404_abs.starts_with(directory) {
        return None;
    }

    build_response(&fallback, StatusCode::NOT_FOUND, &encodings).await
}

async fn build_response(
    filepath: &PathBuf,
    status_code: StatusCode,
    encodings: &Option<Vec<String>>,
) -> Option<Result<BoxedResponse, hyper::http::Error>> {
    let content_type = get_content_type(&filepath);

    // encodings
    if let Some(res) =
        compose_encoded_response(&filepath, content_type, status_code, &encodings).await
    {
        return Some(res);
    };

    // origin target
    compose_get_response(&filepath, content_type, status_code, None).await
}

async fn compose_encoded_response(
    filepath: &PathBuf,
    content_type: &str,
    status_code: StatusCode,
    encodings: &Option<Vec<String>>,
) -> Option<Result<BoxedResponse, hyper::http::Error>> {
    let encds = match encodings {
        Some(encds) => encds,
        _ => return None,
    };

    for enc in encds {
        if let Some(encoded_path) = add_extension(filepath, &enc) {
            if let Some(res) =
                compose_get_response(&encoded_path, content_type, status_code, Some(enc)).await
            {
                return Some(res);
            }
        };
    }

    None
}

async fn compose_get_response(
    filepath: &PathBuf,
    content_type: &str,
    status_code: StatusCode,
    content_encoding: Option<&str>,
) -> Option<Result<BoxedResponse, hyper::http::Error>> {
    let metadata = match fs::metadata(filepath).await {
        Ok(m) => m,
        _ => return None,
    };

    if !metadata.is_file() {
        return None;
    }

    let file = match File::open(filepath).await {
        Ok(m) => m,
        _ => return None,
    };

    let mut builder = Response::builder()
        .status(status_code)
        .header(CONTENT_TYPE, content_type)
        .header(CONTENT_LENGTH, metadata.len());

    if let Some(enc) = content_encoding {
        builder = builder.header(CONTENT_ENCODING, enc);
    }

    // https://github.com/hyperium/hyper/blob/master/examples/send_file.rs
    let reader_stream = ReaderStream::new(file);
    let stream_body = StreamBody::new(reader_stream.map_ok(Frame::data));
    let boxed_body = stream_body.boxed();

    Some(builder.body(boxed_body))
}
