use hyper::body::Incoming;
use hyper::header::ACCEPT_ENCODING;
use hyper::http::Request;
use std::ffi::OsString;
use std::path::PathBuf;
use tokio::fs;

use crate::available_encodings::{get_encoded_ext, AvailableEncodings};

pub async fn get_path_from_request_url(
    req: &Request<Incoming>,
    directory: &PathBuf,
) -> Option<PathBuf> {
    let mut uri_path = req.uri().path().to_string();
    if uri_path.ends_with("/") {
        uri_path.push_str("index.html");
    }

    let stripped = match uri_path.strip_prefix("/") {
        Some(p) => p,
        _ => &uri_path,
    };

    get_path(directory, &PathBuf::from(stripped)).await
}

pub async fn get_path(directory: &PathBuf, filepath: &PathBuf) -> Option<PathBuf> {
    let joined = directory.join(filepath);

    // https://doc.rust-lang.org/std/path/struct.Path.html#method.normalize_lexically
    // normalize lexically in nightly
    let target_path = match fs::canonicalize(joined).await {
        Ok(pb) => pb,
        _ => return None,
    };

    match target_path.starts_with(directory) {
        true => Some(target_path),
        _ => None,
    }
}

pub fn get_encodings(
    req: &Request<Incoming>,
    available_encodings: &AvailableEncodings,
) -> Vec<String> {
    let mut encodings = Vec::new();

    let accept_encoding_header = match req.headers().get(ACCEPT_ENCODING) {
        Some(enc) => enc,
        _ => return encodings,
    };

    let encoding_str = match accept_encoding_header.to_str() {
        Ok(s) => s,
        _ => return encodings,
    };

    for encoding in encoding_str.split(",") {
        let trimmed = encoding.trim();
        if available_encodings.encoding_is_available(trimmed) {
            encodings.push(trimmed.to_string());
        }
    }

    return encodings;
}

// nightly API replacement
// https://doc.rust-lang.org/std/path/struct.Path.html#method.with_added_extension

// Filepath must be a file, not a directory for this to work.
pub fn add_extension(filepath: &PathBuf, encoding: &str) -> Option<PathBuf> {
    let enc_ext = match get_encoded_ext(encoding) {
        Some(enc) => enc,
        _ => return None,
    };

    let mut fp_with_ext = OsString::from(filepath);
    fp_with_ext.push(enc_ext);

    Some(PathBuf::from(fp_with_ext))
}
