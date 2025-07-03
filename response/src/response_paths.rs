use hyper::body::Incoming;
use hyper::header::ACCEPT_ENCODING;
use hyper::http::Request;
use std::ffi::OsStr;
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

    let mut target_path = match path::absolute(directory.join(&stripped)) {
        Ok(pb) => pb,
        _ => return None,
    };

    // confirm path resides in directory
    if !target_path.starts_with(directory) {
        return None;
    }

    let mtdt = match fs::metadata(&target_path).await {
        Ok(sdf) => sdf,
        _ => return None,
    };

    // if file bail early
    if mtdt.is_file() {
        return Some(target_path);
    }

    // if directory try an index.html file
    if mtdt.is_dir() {
        target_path.push("index.html");
    }

    let mtdt = match fs::metadata(&target_path).await {
        Ok(sdf) => sdf,
        _ => return None,
    };

    if mtdt.is_file() {
        return Some(target_path);
    }

    None
}

pub fn get_encodings(
    req: &Request<Incoming>,
    content_encodings: &Option<Vec<String>>,
) -> Option<Vec<String>> {
    let accept_encoding_header = match req.headers().get(ACCEPT_ENCODING) {
        Some(enc) => enc,
        _ => return None,
    };

    let encoding_str = match accept_encoding_header.to_str() {
        Ok(s) => s,
        _ => return None,
    };

    let available_encodings = AvailableEncodings::from(content_encodings);

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

// Filepath must be an file, not a directory for this to work.
pub fn add_extension(filepath: &PathBuf, encoding: &str) -> Option<PathBuf> {
    let enc_ext = match get_encoded_ext(encoding) {
        Some(enc) => enc,
        _ => return None,
    };

    let os_ext = OsStr::new(enc_ext);

    let mut fp_with_ext = filepath.as_os_str().to_os_string();
    fp_with_ext.push(os_ext);

    Some(PathBuf::from(fp_with_ext))
}
