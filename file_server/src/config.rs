use serde::{Deserialize, Serialize};
use serde_json;
use std::env;
use std::path;
use std::path::PathBuf;
use tokio::fs;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Config {
    pub host_and_port: String,
    pub directory: PathBuf,
    pub content_encodings: Option<Vec<String>>,
    pub filepath_404: Option<PathBuf>,
}

impl Config {
    pub fn new() -> Result<Config, String> {
        let curr_dir = match env::current_dir() {
            Ok(pb) => pb,
            Err(e) => return Err(e.to_string()),
        };

        Ok(Config {
            host_and_port: "0.0.0.0:3000".to_string(),
            directory: curr_dir,
            content_encodings: None,
            filepath_404: None,
        })
    }

    pub async fn try_from(filepath: &PathBuf) -> Result<Config, String> {
        // see if config exists
        let config_json = match fs::read_to_string(filepath).await {
            Ok(r) => r,
            Err(e) => return Err(e.to_string()),
        };

        let mut config: Config = match serde_json::from_str(&config_json) {
            Ok(j) => j,
            Err(e) => return Err(e.to_string()),
        };

        // get target directory
        let config_path = match path::absolute(&filepath) {
            Ok(pb) => pb,
            Err(e) => return Err(e.to_string()),
        };

        let parent_dir = match config_path.parent() {
            Some(p) => p,
            _ => {
                return Err("parent directory of config not found".to_string());
            }
        };

        // https://doc.rust-lang.org/std/path/struct.Path.html#method.normalize_lexically
        // normalize lexically
        let target_directory = match fs::canonicalize(parent_dir.join(config.directory)).await {
            Ok(pb) => pb,
            Err(e) => return Err(e.to_string()),
        };

        if let Some(origin_404s) = config.filepath_404 {
            config.filepath_404 =
                match get_path_relative_to_origin(&target_directory, &origin_404s).await {
                    Ok(pb) => Some(pb),
                    Err(e) => return Err(e.to_string()),
                };
        }

        config.directory = target_directory;

        Ok(config)
    }
}

async fn get_path_relative_to_origin(
    source_dir: &PathBuf,
    filepath: &PathBuf,
) -> Result<PathBuf, String> {
    let target_path = source_dir.join(filepath);
    let target_path_abs = match fs::canonicalize(target_path).await {
        Ok(pb) => pb,
        Err(e) => return Err(e.to_string()),
    };

    if target_path_abs.starts_with(source_dir) {
        return Ok(target_path_abs);
    }

    Err("filepath_404 does not reside in source_dir".to_string())
}
