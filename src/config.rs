use serde::{Deserialize, Serialize};
use serde_json;
use tokio::fs;

use std::path;
use std::path::{Path, PathBuf};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Config {
    pub host_and_port: String,
    pub directory: PathBuf,
    pub content_encodings: Vec<String>,
    pub filepath_404s: Vec<(PathBuf, Option<String>)>,
}

impl Config {
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

        let updated_404s = match get_paths(parent_dir, config.filepath_404s) {
            Ok(pb) => pb,
            Err(e) => return Err(e.to_string()),
        };

        config.directory = target_directory_abs;
        config.filepath_404s = updated_404s;

        Ok(config)
    }
}

fn get_paths(
    source_dir: &Path,
    filepaths: Vec<(PathBuf, Option<String>)>,
) -> Result<Vec<(PathBuf, Option<String>)>, String> {
    let mut updated_filepaths = Vec::new();

    for (file_path, encoding_type) in filepaths {
        let target_path = source_dir.join(file_path);
        let target_path_abs = match path::absolute(target_path) {
            Ok(pb) => pb,
            Err(e) => return Err(e.to_string()),
        };
        updated_filepaths.push((target_path_abs, encoding_type));
    }

    Ok(updated_filepaths)
}
