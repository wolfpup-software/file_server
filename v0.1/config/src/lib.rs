use std::env;
use std::fmt;
use std::fs;
use std::path;

use serde_json;
use serde::{Serialize, Deserialize};


static CURR_DIR_NOT_FOUND: &str = "could not find working directory";
static CONFIG_NOT_FOUND_ERR: &str = "no config parameters were found at location";

static JSON_FILE_ERR: &str = "config json file failed to load";
static JSON_SERIALIZE_FAILED_ERR: &str = "config json serialization failed";
static JSON_DESERIALIZE_FAILED_ERR: &str = "config json deserialization failed";

static PARENT_NOT_FOUND_ERR: &str = "parent directory of config not found";

static DIR_TARGET_NOT_FOUND_ERR: &str = "directory target was not found";
static DIR_IS_NOT_DIR_ERR: &str = "config.directory is not a directory";

static FILE_403_NOT_FOUND_ERR: &str = "config.filepath_403 was not found";
static FILE_403_OUT_OF_BOUNDS_ERR: &str = "config.filepath_403 is not located in the base directory";
static FILE_404_NOT_FOUND_ERR: &str = "config.filepath_403 was not found";
static FILE_404_OUT_OF_BOUNDS_ERR: &str = "config.filepath_404 is not located in the base directory";
static FILE_500_NOT_FOUND_ERR: &str = "config.filepath_500 was not found";
static FILE_500_OUT_OF_BOUNDS_ERR: &str = "config.filepath_500 is not located in the base directory";


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
    pub directory: path::PathBuf,
    pub port: u16,
    pub filepath_403: path::PathBuf,
    pub filepath_404: path::PathBuf,
    pub filepath_500: path::PathBuf,
}


pub fn get_file_pathbuff(
	dir: &path::Path,
	filepath: &path::PathBuf,
) -> Result<path::PathBuf, std::io::Error> {
    let fp = path::PathBuf::from(&filepath);
    if fp.has_root() {
        return Ok(fp);
    }

    dir.join(fp).canonicalize()
}

pub fn get_config_pathbuff(
	dir: path::PathBuf,
	filepath: &str,
) -> Result<path::PathBuf, std::io::Error> {
    let fp = path::PathBuf::from(&filepath);
    if fp.has_root() {
        return Ok(fp);
    }

    dir.join(fp).canonicalize()
}

fn validate_filepath(
	rel_dir: &path::Path,
	pb: &path::PathBuf,
	base_dir: &path::PathBuf,
	not_found_err: &str,
	bound_err: &str,
) -> Result<path::PathBuf, ConfigError> {
    let fp = match get_file_pathbuff(rel_dir, pb) {
        Ok(j) => j,
        _ => return Err(ConfigError::new(not_found_err)),
    };
    if !fp.starts_with(base_dir) {
        return Err(ConfigError::new(bound_err));
    }
    
    Ok(fp)
}

pub fn get_config(filepath: &str) -> Result<Config, ConfigError> {
	// get json from filepath
    let curr_dir = match env::current_dir() {
        Ok(pb) => pb,
        _ => return Err(ConfigError::new(CURR_DIR_NOT_FOUND))
    };
    
    let config_pathbuff = match get_config_pathbuff(curr_dir, filepath) {
        Ok(pb) => pb,
        _ => return Err(ConfigError::new(CONFIG_NOT_FOUND_ERR)),
    };

    let json_as_str = match fs::read_to_string(&config_pathbuff) {
        Ok(r) => r,
        _ => return Err(ConfigError::new(JSON_FILE_ERR)),
    };

	// build config from json string
    let config: Config = match serde_json::from_str(&json_as_str) {
        Ok(j) => j,
        _ => return Err(ConfigError::new(JSON_DESERIALIZE_FAILED_ERR)),
    };
    
    let parent = match config_pathbuff.parent() {
    	Some(p) => p,
    	_ => return Err(ConfigError::new(PARENT_NOT_FOUND_ERR))
    };
    
    let directory = match get_file_pathbuff(&parent, &config.directory) {
        Ok(j) => j,
        _ => return Err(ConfigError::new(DIR_TARGET_NOT_FOUND_ERR)),
    };
    if !directory.is_dir() {
        return Err(ConfigError::new(DIR_IS_NOT_DIR_ERR))
    }

	// create config with absolute filepaths from client config
    let filepath_403 = match validate_filepath(
    	&parent,
    	&config.filepath_403,
    	&directory,
    	FILE_403_NOT_FOUND_ERR,
    	FILE_403_OUT_OF_BOUNDS_ERR,
    ) {
        Ok(j) => j,
        Err(e) => return Err(e),
    };
    
    let filepath_404 = match validate_filepath(
    	&parent,
    	&config.filepath_404,
    	&directory,
    	FILE_404_NOT_FOUND_ERR,
    	FILE_404_OUT_OF_BOUNDS_ERR,
    ) {
        Ok(j) => j,
        Err(e) => return Err(e),
    };
    
    let filepath_500 = match validate_filepath(
    	&parent,
    	&config.filepath_500,
    	&directory,
    	FILE_500_NOT_FOUND_ERR,
    	FILE_500_OUT_OF_BOUNDS_ERR,
    ) {
        Ok(j) => j,
        Err(e) => return Err(e),
    };
    
    Ok(Config {
    	directory: directory,
    	port: config.port,
    	filepath_403: filepath_403,
    	filepath_404: filepath_404,
    	filepath_500: filepath_500,
    })
}

pub fn config_to_string(config: &Config) -> Result<String, ConfigError> {
    match serde_json::to_string(config) {
        Ok(s) => Ok(s),
        _ => Err(ConfigError::new(JSON_SERIALIZE_FAILED_ERR))
    }
}