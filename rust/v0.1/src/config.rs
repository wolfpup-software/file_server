use serde::{Deserialize};
use serde_json;
use std::error;
use std::fs;
use std::env;
use std::path;
use std::fmt;

#[derive(Debug)]
pub struct ConfigError {
    message: String,
}

impl ConfigError {
    pub fn new(message: String) -> ConfigError {
        ConfigError { message }
    }
    
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

// make this copyable

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub dir: String,
    pub port: u16,
    pub filepath_400: String,
    pub filepath_404: String,
}

#[derive(Clone, Debug)]
pub struct ConfigBuffs {
    pub dir: path::PathBuf,
    pub port: u16,
    pub filepath_400: path::PathBuf,
    pub filepath_404: path::PathBuf,
}

// impl ConfigBuffs {
//     pub async fn send_file(_req: Request<Body>) {
//         // maybe send file here?
//     }
// }


fn valid_path(base_dir: &path::PathBuf, request_path: &path::PathBuf) -> bool {
    !request_path.starts_with(base_dir)
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
    // here we can move 
    let json_str = match fs::read_to_string(pathbuff) {
        Ok(r) => r,
        Err(_) => return Err(ConfigError::new("config json file failed to load".to_string())),
    };

    match serde_json::from_str(&json_str) {
        Ok(j) => Ok(j),
        Err(_) => Err(ConfigError::new("config json deserialization failed".to_string())),
    }
}

// pub fn get_config(pathbuff: path::PathBuf) -> Result<ConfigBuffs, ConfigError> {
//     // here we can move 
//     let json_str = match fs::read_to_string(pathbuff) {
//         Ok(r) => r,
//         Err(_) => return Err(ConfigError::new("config json file failed to load".to_string())),
//     };

//     let json: Config = match serde_json::from_str(&json_str) {
//         Ok(j) => j,
//         Err(_) => return Err(ConfigError::new("config json deserialization failed".to_string())),
//     };

//     let dir_pathbuff = match get_pathbuff(&json.dir) {
//         Ok(j) => j,
//         Err(_) => return Err(ConfigError::new("base dir does not exist".to_string())),
//     };
//     if !dir_pathbuff.is_dir() {
//         return Err(ConfigError::new("config.dir is not a directory".to_string()))
//     }

//     let pathbuff_400 = match get_pathbuff(&json.filepath_400) {
//         Ok(j) => j,
//         Err(_) => return Err(ConfigError::new("400 html does not exist".to_string())),
//     };
//     if !pathbuff_400.is_file() {
//         return Err(ConfigError::new("config.filepath_400 is not a file".to_string()))
//     }
//     if !valid_path(&dir_pathbuff, &pathbuff_400) {
//         return Err(ConfigError::new("config.filepath_400 is not located in the base directory".to_string()))
//     }

//     let pathbuff_404 = match get_pathbuff(&json.filepath_404) {
//         Ok(j) => j,
//         Err(_) => return Err(ConfigError::new("404 html does not exist".to_string())),
//     };
//     if !pathbuff_404.is_file() {
//         return Err(ConfigError::new("config.filepath_404 is not a file".to_string()))
//     }
//     if !valid_path(&dir_pathbuff, &pathbuff_400) {
//         return Err(ConfigError::new("config.filepath_404 is not located in the base directory".to_string()))
//     }

//     Ok(ConfigBuffs {
//         dir: dir_pathbuff,
//         port: json.port,
//         filepath_400: pathbuff_400,
//         filepath_404: pathbuff_404,
//     })
// }

// pub fn coerce_static_config<'a>(config: ConfigBuffs) -> &'a ConfigBuffs {
//     &config
// }