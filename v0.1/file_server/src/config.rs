use std::fmt;
use std::fs;
use std::path;

use serde::{Deserialize, Serialize};
use serde_json;

const PARENT_NOT_FOUND_ERR: &str = "parent directory of config not found";
const DIR_IS_NOT_DIR_ERR: &str = "config.directory is not a directory";

pub enum ConfigError<'a> {
    IoError(std::io::Error),
    JsonError(serde_json::Error),
    GenericError(&'a str),
}

impl fmt::Display for ConfigError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConfigError::IoError(io_error) => write!(f, "{}", io_error),
            ConfigError::JsonError(json_error) => write!(f, "{}", json_error),
            ConfigError::GenericError(generic_error) => write!(f, "{}", generic_error),
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub directory: path::PathBuf,
}

pub fn from_filepath(filepath: &path::PathBuf) -> Result<Config, ConfigError> {
    // get position relative to working directory
    let config_pathbuff = match filepath.canonicalize() {
        Ok(pb) => pb,
        Err(e) => return Err(ConfigError::IoError(e)),
    };

    let parent_dir = match config_pathbuff.parent() {
        Some(p) => p,
        _ => return Err(ConfigError::GenericError(PARENT_NOT_FOUND_ERR)),
    };

    let json_reader = match fs::File::open(&config_pathbuff) {
        Ok(r) => r,
        Err(e) => return Err(ConfigError::IoError(e)),
    };

    // update the directory relative to config filepath
    let mut config: Config = match serde_json::from_reader(&json_reader) {
        Ok(j) => j,
        Err(e) => return Err(ConfigError::JsonError(e)),
    };

    let directory = match parent_dir.join(&config.directory).canonicalize() {
        Ok(j) => j,
        Err(e) => return Err(ConfigError::IoError(e)),
    };
    if !directory.is_dir() {
        return Err(ConfigError::GenericError(DIR_IS_NOT_DIR_ERR));
    }

    config.directory = directory;

    Ok(config)
}
