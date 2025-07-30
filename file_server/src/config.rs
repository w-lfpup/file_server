use serde::{Deserialize, Serialize};
use serde_json;
use std::env;
use std::path;
use std::path::{Path, PathBuf};
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

    pub async fn try_from(source_path: &PathBuf) -> Result<Config, String> {
        // see if config exists
        let config_json = match fs::read_to_string(source_path).await {
            Ok(r) => r,
            Err(e) => return Err(e.to_string()),
        };

        let mut config: Config = match serde_json::from_str(&config_json) {
            Ok(j) => j,
            Err(e) => return Err(e.to_string()),
        };

        // get target directory
        let config_path = match path::absolute(&source_path) {
            Ok(pb) => pb,
            Err(e) => return Err(e.to_string()),
        };

        let parent_dir = match config_path.parent() {
            Some(p) => p,
            _ => {
                return Err("parent directory of config not found".to_string());
            }
        };

        // get target directory relative to config path
        let target_directory = parent_dir.join(config.directory);
        let target_directory_abs = match path::absolute(target_directory) {
            Ok(pb) => pb,
            Err(e) => return Err(e.to_string()),
        };

        config.directory = target_directory_abs;

        if let Some(origin_404s) = config.filepath_404 {
            config.filepath_404 = match get_path_relative_to_origin(parent_dir, &origin_404s) {
                Ok(pb) => Some(pb),
                Err(e) => return Err(e.to_string()),
            };
        }

        Ok(config)
    }
}

fn get_path_relative_to_origin(source_dir: &Path, filepath: &PathBuf) -> Result<PathBuf, String> {
    let target_path = source_dir.join(filepath);
    let target_path_abs = match path::absolute(target_path) {
        Ok(pb) => pb,
        Err(e) => return Err(e.to_string()),
    };

    if target_path_abs.starts_with(source_dir) {
        return Ok(target_path_abs);
    }

    Err("filepath_404 does not reside in source_dir".to_string())
}
