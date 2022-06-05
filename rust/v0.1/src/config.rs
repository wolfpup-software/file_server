use std::env;
use std::fmt;
use std::fs;
use std::path;

use serde_json;
use serde::{Deserialize};

static JSON_FILE_ERR: &str = "config json file failed to load";
static JSON_DESERIALIZE_ERR: &str = "config json deserialization failed";
static DIR_IS_NOT_DIR_ERR: &str = "config.dir is not a directory";
static OUT_OF_BOUNDS_403_ERR: &str = "config.filepath_403 is not located in the base directory";
static NOT_A_FILE_403_ERR: &str = "config.filepath_403 is not a file";
static OUT_OF_BOUNDS_404_ERR: &str = "config.filepath_404 is not located in the base directory";
static NOT_A_FILE_404_ERR: &str = "config.filepath_404 is not a file";
static OUT_OF_BOUNDS_500_ERR: &str = "config.filepath_500 is not located in the base directory";
static NOT_A_FILE_500_ERR: &str = "config.filepath_500 is not a file";

pub struct ConfigError {
    message: String,
}

impl ConfigError {
    pub fn new(msg: &str) -> ConfigError {
        ConfigError { message: msg.to_string() }
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

#[derive(Clone, Deserialize)]
pub struct Config {
    pub dir: path::PathBuf,
    pub port: u16,
    pub filepath_403: path::PathBuf,
    pub filepath_404: path::PathBuf,
    pub filepath_500: path::PathBuf,
}

fn valid_path(base_dir: &path::PathBuf, request_path: &path::PathBuf) -> bool {
    request_path.starts_with(base_dir)
}

pub fn get_pathbuff(config_filepath: &str) -> Result<path::PathBuf, std::io::Error> {
    let filepath = path::PathBuf::from(&config_filepath);
    if filepath.has_root() {
        return Ok(filepath);
    }

    let curr_dir = match env::current_dir() {
        Ok(pb) => pb,
        Err(e) => return Err(e),
    };

    curr_dir.join(filepath).canonicalize()
}

pub fn get_config(pathbuff: path::PathBuf) -> Result<Config, ConfigError> {
    let json_str = match fs::read_to_string(pathbuff) {
        Ok(r) => r,
        Err(_) => return Err(ConfigError::new(JSON_FILE_ERR)),
    };

    let config: Config = match serde_json::from_str(&json_str) {
        Ok(j) => j,
        Err(_) => return Err(ConfigError::new(JSON_DESERIALIZE_ERR)),
    };
    if !config.dir.is_dir() {
        return Err(ConfigError::new(DIR_IS_NOT_DIR_ERR))
    }

    if !config.filepath_404.is_file() {
        return Err(ConfigError::new(NOT_A_FILE_404_ERR))
    }
    if !valid_path(&config.dir, &config.filepath_404) {
        return Err(ConfigError::new(OUT_OF_BOUNDS_404_ERR))
    }

    if !config.filepath_403.is_file() {
        return Err(ConfigError::new(NOT_A_FILE_403_ERR))
    }
    if !valid_path(&config.dir, &config.filepath_403) {
        return Err(ConfigError::new(OUT_OF_BOUNDS_403_ERR))
    }

    if !config.filepath_500.is_file() {
        return Err(ConfigError::new(NOT_A_FILE_500_ERR))
    }
    if !valid_path(&config.dir, &config.filepath_500) {
        return Err(ConfigError::new(OUT_OF_BOUNDS_500_ERR))
    }

    Ok(config)
}
