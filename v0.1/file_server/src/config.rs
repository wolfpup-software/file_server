use std::fmt;
use tokio::fs;
use std::path;

use serde::{Deserialize, Serialize};
use serde_json;

pub enum ConfigError<'a> {
    IoError(std::io::Error),
    JsonError(serde_json::Error),
    Error(&'a str),
}

impl fmt::Display for ConfigError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConfigError::IoError(io_error) => write!(f, "{}", io_error),
            ConfigError::JsonError(json_error) => write!(f, "{}", json_error),
            ConfigError::Error(error_str) => write!(f, "{}", error_str),
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub directory: path::PathBuf,
}

pub async fn from_filepath(filepath: &path::PathBuf) -> Result<Config, ConfigError> {
    // get position relative to working directory
    let config_pathbuff = match filepath.canonicalize() {
        Ok(pb) => pb,
        Err(e) => return Err(ConfigError::IoError(e)),
    };
    let parent_dir = match config_pathbuff.parent() {
        Some(p) => p,
        _ => return Err(ConfigError::Error("parent directory of config file does not exist.")),
    };

    let config_json = match fs::read_to_string(&config_pathbuff).await {
        Ok(r) => r,
        Err(e) => return Err(ConfigError::IoError(e)),
    };

    let mut config: Config = match serde_json::from_str(&config_json) {
        Ok(j) => j,
        Err(e) => return Err(ConfigError::JsonError(e)),
    };
    
    // update the directory relative to config filepath
    let directory = match parent_dir.join(&config.directory).canonicalize() {
        Ok(j) => j,
        Err(e) => return Err(ConfigError::IoError(e)),
    };
    if !directory.is_dir() {
        return Err(ConfigError::Error("config.directory is not a directory"));
    }

    config.directory = directory;

    Ok(config)
}
