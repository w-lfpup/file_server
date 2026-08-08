use futures_util::TryStreamExt;
use http_body_util::{BodyExt, Full, StreamBody};
use hyper::body::{Frame, Incoming};
use hyper::header::{HeaderValue, CONTENT_ENCODING, CONTENT_LENGTH, CONTENT_TYPE};
use hyper::http::{Request, Response};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ffi::OsString;
use std::fs::Metadata;
use std::path::PathBuf;
use tokio::fs;
use tokio_util::io::ReaderStream;
// use tokio_stream::{StreamExt, wrappers::ReadDirStream};
use serde_json;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::content_type::get_content_type;
use crate::last_resort_response;
use crate::range_response;
use crate::response_paths::{add_extension, get_encodings};
use crate::type_flyweight::{BoxedResponse, ResponseParams, NOT_FOUND_404};

#[derive(Clone, Serialize, Deserialize, Debug)]
struct EntryDetails {
    is_dir: bool,
    file_name: String,
    url_path: PathBuf,
    size: u64,
    created_at: Option<SystemTime>,
    accessed_at: Option<SystemTime>,
    modified_at: Option<SystemTime>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
struct Details {
    r#type: String,
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

    // we have a query
    let params_map = form_urlencoded::parse(params.as_bytes())
        .into_owned()
        .collect::<HashMap<String, String>>();

    let serialize_as = match params_map.get("details_as") {
        Some(srlz) => srlz,
        _ => return None,
    };

    println!("got some json!\n {:?}", serialize_as);

    if "json" != serialize_as {
        return None;
    }

    // need to leave it with / to get file details
    // otherwise get_path_from_request_url always returns from /index.html

    let req_path = match get_path_from_request_url(req, &res_params.directory).await {
        Some(pth) => pth,
        _ => return None,
    };
    println!("{:?}", req_path);

    let metadata = match fs::metadata(&req_path).await {
        Ok(m) => m,
        // return 404
        _ => return None,
    };
    println!("{:?}", metadata);

    if metadata.is_symlink() {
        // 404
        return None;
    }

    // this could just be one function that splits into two later
    let details = match metadata.is_dir() {
        true => build_directory_entry(&metadata, &req_path, &res_params.directory).await,
        _ => build_file_entry(&metadata, &req_path, &res_params.directory),
    };

    if let Some(deets) = details {
        if let Some(res) = compose_response(&deets).await {
            return Some(res);
        }
    }

    Some(last_resort_response::build_response(
        StatusCode::NOT_FOUND,
        NOT_FOUND_404,
    ))
}

fn build_file_entry(
    metadata: &Metadata,
    req_path: &PathBuf,
    base_path: &PathBuf,
) -> Option<Details> {
    let mut details = Details {
        r#type: "file".to_string(),
        entries: Vec::new(),
    };

    details
        .entries
        .push(create_entry_details(metadata, req_path, base_path));

    Some(details)
}

async fn build_directory_entry(
    metadata: &Metadata,
    req_path: &PathBuf,
    base_path: &PathBuf,
) -> Option<Details> {
    let mut details = Details {
        r#type: "directory".to_string(),
        entries: Vec::new(),
    };

    let mut entries = match fs::read_dir(req_path).await {
        Ok(entrs) => entrs,
        _ => return None,
    };

    while true {
        if let Ok(opt_entry) = entries.next_entry().await {
            if let Some(entry) = opt_entry {
                details
                    .entries
                    .push(create_entry_details(metadata, req_path, base_path));
                continue;
            }
        }
        break;
    }
    // while let Ok(opt_entry) = entries.next_entry().await {
    //     if let Some(entry) = opt_entry {
    //         details
    //             .entries
    //             .push(create_entry_details(metadata, req_path, base_path));
    //     }
    // }

    Some(details)
}

fn create_entry_details(
    metadata: &Metadata,
    req_path: &PathBuf,
    base_path: &PathBuf,
) -> EntryDetails {
    let file_name = match req_path.file_name() {
        Some(flnm) => flnm.to_string_lossy().to_string(),
        None => "".to_string(),
    };

    // Some(created.duration_since(UNIX_EPOCH)
    // since_the_epoch.as_millis()
    let created_at = match metadata.created() {
        Ok(created) => Some(created),
        _ => None,
    };
    let accessed_at = match metadata.accessed() {
        Ok(accessed) => Some(accessed),
        _ => None,
    };
    let modified_at = match metadata.modified() {
        Ok(modifed) => Some(modifed),
        _ => None,
    };

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

//

async fn compose_response(details: &Details) -> Option<Result<BoxedResponse, hyper::http::Error>> {
    // details to string
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

pub async fn get_path_from_request_url(
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

    println!("{:?}", target_path);
    match target_path.starts_with(directory) {
        true => Some(target_path),
        _ => None,
    }
}
