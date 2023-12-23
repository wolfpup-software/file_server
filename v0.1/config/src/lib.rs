use std::env;
use std::fmt;
use std::fs;
use std::path;

use serde_json;
use serde::{Serialize, Deserialize};

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

impl Config {
  pub fn from_filepath(filepath: &path::PathBuf) -> Result<Config, ConfigError> {
    // get position relative to working directory
    let working_dir = match env::current_dir() {
      Ok(pb) => pb,
      Err(e) => return Err(ConfigError::IoError(e))
    };
    
    let config_pathbuff = match combine_pathbuff(&working_dir.to_path_buf(), filepath) {
      Ok(pb) => pb,
      Err(e) => return Err(ConfigError::IoError(e))
    };

    let parent_dir = match config_pathbuff.parent() {
      Some(p) => p.to_path_buf(),
      _ =>  return Err(ConfigError::GenericError(PARENT_NOT_FOUND_ERR))
    };

    let json_as_str = match fs::read_to_string(&config_pathbuff) {
      Ok(r) => r,
      Err(e) => return Err(ConfigError::IoError(e))
    };

    let config: Config = match serde_json::from_str(&json_as_str) {
      Ok(j) => j,
      Err(e) => return Err(ConfigError::JsonError(e))
    };
    
    let directory = match combine_pathbuff(&parent_dir, &config.directory) {
      Ok(j) => j,
      Err(e) =>  return Err(ConfigError::IoError(e))
    };
    if !directory.is_dir() {
      return Err(ConfigError::GenericError(DIR_IS_NOT_DIR_ERR))
    }
    
    Ok(Config {
      host: config.host,
      port: config.port,
      directory: directory,
    })
  }
}

fn combine_pathbuff(
	base_dir: &path::PathBuf,
	filepath: &path::PathBuf,
) -> Result<path::PathBuf, std::io::Error> {
  base_dir.join(filepath).canonicalize()
}

