use std::env;
use std::fmt;
use std::fs;
use std::path;

use serde_json;
use serde::{Deserialize};

static JSON_FILE_ERR: &str = "config json file failed to load";
static JSON_DESERIALIZE_ERR: &str = "config json deserialization failed";
static DIR_ERR: &str = "base dir does not exist";
static DIR_IS_NOT_DIR_ERR: &str = "config.dir is not a directory";
static DOES_NOT_EXIST_403_ERR: &str = "config.filepath_403 does not exist";
static OUT_OF_BOUNDS_403_ERR: &str = "config.filepath_403 is not located in the base directory";
static NOT_A_FILE_403_ERR: &str = "config.filepath_403 is not a file";
static DOES_NOT_EXIST_404_ERR: &str = "config.filepath_404 does not exist";
static OUT_OF_BOUNDS_404_ERR: &str = "config.filepath_404 is not located in the base directory";
static NOT_A_FILE_404_ERR: &str = "config.filepath_404 is not a file";

#[derive(Debug)]
pub struct ConfigError<'a> {
    message: &'a str,
}

impl<'a> ConfigError<'a> {
    pub fn new(message: &str) -> ConfigError {
        ConfigError { message }
    }
    
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub dir: String,
    pub port: u16,
    pub filepath_403: String,
    pub filepath_404: String,
}

#[derive(Clone, Debug)]
pub struct ConfigBuffs {
    pub dir: path::PathBuf,
    pub filepath_403: path::PathBuf,
    pub filepath_404: path::PathBuf,
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

pub fn get_config(pathbuff: path::PathBuf) -> Result<Config, ConfigError<'static>> {
    let json_str = match fs::read_to_string(pathbuff) {
        Ok(r) => r,
        Err(_) => return Err(ConfigError::new(JSON_FILE_ERR)),
    };

    match serde_json::from_str(&json_str) {
        Ok(j) => Ok(j),
        Err(_) => Err(ConfigError::new(JSON_DESERIALIZE_ERR)),
    }
}

pub fn get_config_buffs(config: &Config) -> Result<ConfigBuffs, ConfigError<'static>> {
    let dir_pathbuff = match get_pathbuff(&config.dir) {
        Ok(j) => j,
        Err(_) => return Err(ConfigError::new(DIR_ERR)),
    };
    if !dir_pathbuff.is_dir() {
        return Err(ConfigError::new(DIR_IS_NOT_DIR_ERR))
    }

    let pathbuff_403 = match get_pathbuff(&config.filepath_403) {
        Ok(j) => j,
        Err(_) => return Err(ConfigError::new(DOES_NOT_EXIST_403_ERR)),
    };
    if !pathbuff_403.is_file() {
        return Err(ConfigError::new(NOT_A_FILE_403_ERR))
    }
    if !valid_path(&dir_pathbuff, &pathbuff_403) {
        return Err(ConfigError::new(OUT_OF_BOUNDS_403_ERR))
    }

    let pathbuff_404 = match get_pathbuff(&config.filepath_404) {
        Ok(j) => j,
        Err(_) => return Err(ConfigError::new(DOES_NOT_EXIST_404_ERR)),
    };
    if !pathbuff_404.is_file() {
        return Err(ConfigError::new(NOT_A_FILE_404_ERR))
    }
    if !valid_path(&dir_pathbuff, &pathbuff_404) {
        return Err(ConfigError::new(OUT_OF_BOUNDS_404_ERR))
    }

    Ok(ConfigBuffs {
        dir: dir_pathbuff,
        filepath_403: pathbuff_403,
        filepath_404: pathbuff_404,
    })
}
