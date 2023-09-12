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
        
        Ok(Config {
            host: config.host,
            port: config.port,
            directory: directory,
        })
    }
}

pub struct ServiceConfig {
    pub directory: path::PathBuf,
}

impl ServiceConfig {
    pub fn from_config(config: &Config) -> ServiceConfig {
        ServiceConfig {
            directory: config.directory.clone(),
        }
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

