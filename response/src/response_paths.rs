use hyper::body::Incoming;
use hyper::header::ACCEPT_ENCODING;
use hyper::http::Request;
use std::path::PathBuf;

use crate::available_encodings::{get_encoded_ext, AvailableEncodings};

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

pub fn add_extension(filepath: &PathBuf, encoding: &str) -> Option<PathBuf> {
    let enc_ext = match get_encoded_ext(encoding) {
        Some(enc) => enc,
        _ => return None,
    };

    let mut ext_path = filepath.clone();
    ext_path.add_extension(enc_ext);

    Some(ext_path)
}
