use std::env;
use std::fmt;
use std::fs;
use std::path;

use serde_json;
use serde::{Serialize, Deserialize};


const CURR_DIR_NOT_FOUND: &str = "could not find working directory";
const CONFIG_NOT_FOUND_ERR: &str = "no config parameters were found at location";

const JSON_FILE_ERR: &str = "config json file failed to load";
const JSON_SERIALIZE_FAILED_ERR: &str = "config json serialization failed";
const JSON_DESERIALIZE_FAILED_ERR: &str = "config json deserialization failed";

const PARENT_NOT_FOUND_ERR: &str = "parent directory of config not found";

const DIR_TARGET_NOT_FOUND_ERR: &str = "directory target was not found";
const DIR_IS_NOT_DIR_ERR: &str = "config.directory is not a directory";

const FILE_404_NOT_FOUND_ERR: &str = "config.filepath_403 was not found";
const FILE_404_OUT_OF_BOUNDS_ERR: &str = "config.filepath_404 is not located in the base directory";
const FILE_500_NOT_FOUND_ERR: &str = "config.filepath_500 was not found";
const FILE_500_OUT_OF_BOUNDS_ERR: &str = "config.filepath_500 is not located in the base directory";


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

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub directory: path::PathBuf,
    pub filepath_404: path::PathBuf,
    pub filepath_500: path::PathBuf,
}

impl Config {
    pub fn from_filepath(filepath: &path::PathBuf) -> Result<Config, ConfigError> {
        // get position relative to working directory
        let working_dir = match env::current_dir() {
            Ok(pb) => pb,
            _ => return Err(ConfigError::new(CURR_DIR_NOT_FOUND))
        };
        
        let config_pathbuff = match combine_pathbuff(&working_dir.to_path_buf(), filepath) {
            Ok(pb) => pb,
            _ => return Err(ConfigError::new(CONFIG_NOT_FOUND_ERR)),
        };

        let parent_dir = match config_pathbuff.parent() {
            Some(p) => p.to_path_buf(),
            _ => return Err(ConfigError::new(PARENT_NOT_FOUND_ERR))
        };
    
        // build json conifg
        let json_as_str = match fs::read_to_string(&config_pathbuff) {
            Ok(r) => r,
            _ => return Err(ConfigError::new(JSON_FILE_ERR)),
        };
    
        let config: Config = match serde_json::from_str(&json_as_str) {
            Ok(j) => j,
            _ => return Err(ConfigError::new(JSON_DESERIALIZE_FAILED_ERR)),
        };
        
        let directory = match combine_pathbuff(&parent_dir, &config.directory) {
            Ok(j) => j,
            _ => return Err(ConfigError::new(DIR_TARGET_NOT_FOUND_ERR)),
        };
        if !directory.is_dir() {
            return Err(ConfigError::new(DIR_IS_NOT_DIR_ERR))
        }
        
        let filepath_404 = match validate_filepath(
            &parent_dir,
            &config.filepath_404,
            &directory,
            FILE_404_NOT_FOUND_ERR,
            FILE_404_OUT_OF_BOUNDS_ERR,
        ) {
            Ok(j) => j,
            Err(e) => return Err(e),
        };
        
        let filepath_500 = match validate_filepath(
            &parent_dir,
            &config.filepath_500,
            &directory,
            FILE_500_NOT_FOUND_ERR,
            FILE_500_OUT_OF_BOUNDS_ERR,
        ) {
            Ok(j) => j,
            Err(e) => return Err(e),
        };
        
        Ok(Config {
            host: config.host,
            port: config.port,
            directory: directory,
            filepath_404: filepath_404,
            filepath_500: filepath_500,
        })
    }
}

pub struct ServiceConfig {
    pub directory: path::PathBuf,
    pub filepath_404: path::PathBuf,
    pub filepath_500: path::PathBuf,
}

impl ServiceConfig {
    pub fn from_config(config: &Config) -> ServiceConfig {
        ServiceConfig {
            directory: config.directory.clone(),
            filepath_404: config.filepath_404.clone(),
            filepath_500: config.filepath_500.clone(),
        }
    }
}

fn validate_filepath(
	parent_dir: &path::PathBuf,
	pb: &path::PathBuf,
	base_dir: &path::PathBuf,
	not_found_err: &str,
	bound_err: &str,
) -> Result<path::PathBuf, ConfigError> {
    match combine_pathbuff(parent_dir, pb) {
        Ok(fp) => {
            if fp.starts_with(base_dir) {
                return Ok(fp);
            }

            Err(ConfigError::new(bound_err))
        },
        _ => return Err(ConfigError::new(not_found_err)),
    }
}

fn combine_pathbuff(
	base_dir: &path::PathBuf,
	filepath: &path::PathBuf,
) -> Result<path::PathBuf, std::io::Error> {
    let mut fp = path::PathBuf::from(&base_dir);
    fp.push(filepath);

    fp.canonicalize()
}

pub fn config_to_string(config: &Config) -> Result<String, ConfigError> {
    match serde_json::to_string(config) {
        Ok(s) => Ok(s),
        _ => Err(ConfigError::new(JSON_SERIALIZE_FAILED_ERR))
    }
}
