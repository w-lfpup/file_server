use hyper::body::Incoming;
use hyper::header::ACCEPT_ENCODING;
use hyper::http::Request;
use std::ffi::OsString;
use std::path;
use std::path::PathBuf;
use tokio::fs;

use crate::available_encodings::{get_encoded_ext, AvailableEncodings};

pub async fn get_path_from_request_url(
    req: &Request<Incoming>,
    directory: &PathBuf,
) -> Option<PathBuf> {
    let uri_path = req.uri().path();

    let stripped = match uri_path.strip_prefix("/") {
        Some(p) => p,
        _ => uri_path,
    };

    get_path(directory, &PathBuf::from(stripped)).await
}

pub async fn get_path(directory: &PathBuf, filepath: &PathBuf) -> Option<PathBuf> {
    let mut target_path = match path::absolute(directory.join(&filepath)) {
        Ok(pb) => pb,
        _ => return None,
    };

    // confirm path resides in directory
    if !target_path.starts_with(directory) {
        return None;
    }

    let metadata = match fs::metadata(&target_path).await {
        Ok(md) => md,
        _ => return None,
    };

    // if file bail early
    if metadata.is_file() {
        return Some(target_path);
    }

    // if directory try an index.html file
    if metadata.is_dir() {
        target_path.push("index.html");

        let updated_metadata = match fs::metadata(&target_path).await {
            Ok(md) => md,
            _ => return None,
        };

        if updated_metadata.is_file() {
            return Some(target_path);
        }
    }

    None
}

pub fn get_encodings(
    req: &Request<Incoming>,
    available_encodings: &AvailableEncodings,
) -> Option<Vec<String>> {
    let accept_encoding_header = match req.headers().get(ACCEPT_ENCODING) {
        Some(enc) => enc,
        _ => return None,
    };

    let encoding_str = match accept_encoding_header.to_str() {
        Ok(s) => s,
        _ => return None,
    };

    let mut encodings = Vec::new();
    for encoding in encoding_str.split(",") {
        let trimmed = encoding.trim();
        if available_encodings.encoding_is_available(trimmed) {
            encodings.push(trimmed.to_string());
        }
    }

    if 0 < encodings.len() {
        return Some(encodings);
    }

    None
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
