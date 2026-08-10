use futures_util::TryStreamExt;
use http_body_util::{BodyExt, Full, StreamBody};
use hyper::body::{Frame, Incoming};
use hyper::header::{HeaderValue, CONTENT_ENCODING, CONTENT_LENGTH, CONTENT_TYPE};
use hyper::http::{Request, Response};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::ffi::OsString;
use std::fs::DirEntry;
use std::fs::Metadata;
use std::io::Error as IoError;
use std::path::Path;
use std::path::PathBuf;
use std::time::{SystemTime, SystemTimeError, UNIX_EPOCH};
use tokio::fs;
use tokio_util::io::ReaderStream;

use crate::content_type::get_content_type;
use crate::last_resort_response;
use crate::range_response;
use crate::response_paths::{add_extension, get_encodings};
use crate::type_flyweight::{BoxedResponse, ResponseParams, NOT_FOUND_404};

// Need to jouge up for some clean json stuff, option should be property exists or no not NULL
#[derive(Clone, Serialize, Deserialize, Debug)]
struct EntryDetails {
    is_dir: bool,
    file_name: String,
    url_path: PathBuf,
    size: u64,
    created_at: Option<u128>,
    accessed_at: Option<u128>,
    modified_at: Option<u128>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
struct FileDetails {
    entry: EntryDetails,
    entries: Vec<EntryDetails>,
}

pub async fn build_response(
    req: &Request<Incoming>,
    res_params: &ResponseParams,
) -> Option<Result<BoxedResponse, hyper::http::Error>> {
    let params = match req.uri().query() {
        Some(prms) => prms,
        _ => return None,
    };

    if 0 == params.len() {
        return None;
    }

    let params_map = form_urlencoded::parse(params.as_bytes())
        .into_owned()
        .collect::<HashMap<String, String>>();

    let serialize_as = match params_map.get("details_as") {
        Some(srlz) => srlz,
        _ => return None,
    };

    if "json" != serialize_as {
        return None;
    }

    if let Some(details) = get_details(req, res_params).await {
        if let Some(res) = compose_response(&details).await {
            return Some(res);
        }
    }

    Some(last_resort_response::build_response(
        StatusCode::NOT_FOUND,
        NOT_FOUND_404,
    ))
}

async fn get_details(req: &Request<Incoming>, res_params: &ResponseParams) -> Option<FileDetails> {
    let req_path = match get_path_from_request_url(req, &res_params.directory).await {
        Some(pth) => pth,
        _ => return None,
    };

    let metadata = match fs::metadata(&req_path).await {
        Ok(m) => m,
        _ => return None,
    };

    // if symlink?

    match metadata.is_dir() {
        true => build_directory_entry(&metadata, &req_path, &res_params.directory).await,
        _ => build_file_entry(&metadata, &req_path, &res_params.directory),
    }
}

async fn get_path_from_request_url(
    req: &Request<Incoming>,
    directory: &PathBuf,
) -> Option<PathBuf> {
    let mut uri_path = req.uri().path().to_string();
    let stripped = match req.uri().path().strip_prefix("/") {
        Some(p) => p,
        _ => &uri_path,
    };

    let joined = directory.join(PathBuf::from(stripped));

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

fn build_file_entry(
    metadata: &Metadata,
    req_path: &PathBuf,
    base_path: &PathBuf,
) -> Option<FileDetails> {
    let entry = create_entry_details(metadata, req_path, base_path);

    let mut details = FileDetails {
        entry,
        entries: Vec::new(),
    };

    Some(details)
}

async fn build_directory_entry(
    metadata: &Metadata,
    req_path: &PathBuf,
    base_path: &PathBuf,
) -> Option<FileDetails> {
    let entry = create_entry_details(metadata, req_path, base_path);

    let mut details = FileDetails {
        entry,
        entries: Vec::new(),
    };

    let mut entries = match fs::read_dir(req_path).await {
        Ok(entrs) => entrs,
        _ => return None,
    };

    while let Ok(opt_entry) = entries.next_entry().await {
        if let Some(entry) = opt_entry {
            let metadata = match entry.metadata().await {
                Ok(md) => md,
                _ => continue,
            };

            details
                .entries
                .push(create_entry_details(&metadata, &entry.path(), base_path));
            continue;
        }

        break;
    }

    Some(details)
}

fn create_entry_details(metadata: &Metadata, req_path: &Path, base_path: &PathBuf) -> EntryDetails {
    let file_name = match req_path.file_name() {
        Some(flnm) => flnm.to_string_lossy().to_string(),
        None => "".to_string(),
    };

    let created_at = get_duration_since_as_ms(metadata.created());
    let accessed_at = get_duration_since_as_ms(metadata.accessed());
    let modified_at = get_duration_since_as_ms(metadata.modified());

    let url_path: PathBuf = match req_path.strip_prefix(base_path) {
        Ok(fp) => fp.to_path_buf(),
        Err(_) => PathBuf::from(""),
    };

    EntryDetails {
        is_dir: metadata.is_dir(),
        file_name,
        url_path,
        created_at,
        accessed_at,
        modified_at,
        size: metadata.len(),
    }
}

async fn compose_response(
    details: &FileDetails,
) -> Option<Result<BoxedResponse, hyper::http::Error>> {
    let body = match serde_json::to_string(details) {
        Ok(bdy) => bdy,
        Err(_) => return None,
    };

    Some(
        Response::builder()
            .status(StatusCode::OK)
            .header(CONTENT_TYPE, HeaderValue::from_static("application/json"))
            .body(
                Full::new(bytes::Bytes::from(body))
                    .map_err(|e| match e {})
                    .boxed(),
            ),
    )
}

fn get_duration_since_as_ms(timestamp_result: Result<SystemTime, IoError>) -> Option<u128> {
    if let Ok(timestamp) = timestamp_result {
        if let Ok(duration) = timestamp.duration_since(SystemTime::UNIX_EPOCH) {
            return Some(duration.as_millis());
        }
    }

    None
}
