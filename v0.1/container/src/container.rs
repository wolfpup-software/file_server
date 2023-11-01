use std::env;
use std::path;
use std::fs;
use std::io::Write;
use std::fmt;

use config;


const CNTR_HOST: &str = "0.0.0.0";
const CTNR_TARGET_DIR: &str = "/usr/share/www";
const FILE_SERVER_JSON_TARGET: &str = "file_server.json";
const PODMAN_COMPOSE_TARGET: &str = "podman-compose.yml";
const PODMANFILE_TARGET: &str = "podmanfile";

const FAILED_TO_CONVERT_CONFIG: &str = "failed to convert config into string";
const FAILED_TO_CREATE_CONFIG: &str = "failed to create config";
const FAILED_TO_WRITE_CONFIG: &str = "failed to write config to disk";

const PODMAN_COMPOSE_NOT_FOUND: &str = "podman-compose template was not found";
const FAILED_TO_CONVERT_PODMAN_COMPOSE: &str = "failed to convert podman-compose template to string";
const FAILED_TO_CREATE_PODMAN_COMPOSE: &str = "failed to create podman-compose";
const FAILED_TO_WRITE_PODMAN_COMPOSE: &str = "failed to wright podman-compose to disk";


// podmanfile should point to file_server/ and confg/ not copy

pub struct ContainerError {
    message: String,
}

impl ContainerError {
    pub fn new(msg: &str) -> ContainerError {
        ContainerError { message: msg.to_string() }
    }
}

impl fmt::Display for ContainerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

pub fn get_pathbuff_from_args(index: usize) -> Option<path::PathBuf> {
    match env::args().nth(index) {
        Some(c) => Some(path::PathBuf::from(c)),
        _ => None,
    }
}

pub fn create_container_config() -> Result<config::Config, ContainerError> {
    let dest = path::PathBuf::from(CTNR_TARGET_DIR);

    Ok(config::Config {
			host: CNTR_HOST.to_string(),
			port: 3000,
			directory: dest,
    })
}

pub fn write_config(
    destination: &path::PathBuf,
    config: &config::Config,
) -> Result<(), ContainerError> {
    let config = match config::config_to_string(&config) {
        Ok(s) => s,
        _ => return Err(ContainerError::new(FAILED_TO_CONVERT_CONFIG)),
    };

    let mut target = path::PathBuf::from(destination);
    target.push(FILE_SERVER_JSON_TARGET);

    let mut output = match fs::File::create(target) {
        Ok(o) => o,
        _ => return Err(ContainerError::new(FAILED_TO_CREATE_CONFIG)),
    };
    
    match output.write_all(config.as_bytes()) {
        Ok(o) => Ok(o),
        _ => return Err(ContainerError::new(FAILED_TO_WRITE_CONFIG)),
    }
}

pub fn write_podmanfile(
    destination: &path::PathBuf,
    podmanfile_filepath: &path::PathBuf,
) -> Result<(), ContainerError> {
  let contents = match fs::read_to_string(podmanfile_filepath) {
      Ok(c) => c,
      _ => return Err(ContainerError::new(PODMAN_COMPOSE_NOT_FOUND)),
  };

	let manifest_dir = match std::env::var("CARGO_MANIFEST_DIR") {
    Ok(c) => c,
    _ => return Err(ContainerError::new(PODMAN_COMPOSE_NOT_FOUND)),
	};
	
	if manifest_dir.len() == 0 {
		return Err(ContainerError::new(PODMAN_COMPOSE_NOT_FOUND));
	}
	
	// import v0.1 entirely
	let manifest_path = path::PathBuf::from(manifest_dir);
	let file_server_path = match manifest_path.join("../").canonicalize() {
		Ok(p) => p,
		_ => return Err(ContainerError::new(PODMAN_COMPOSE_NOT_FOUND)),
	};
	
  let podmanfile = contents
    .replace("{file_server_dir}", &file_server_path.display().to_string());

	let mut target = path::PathBuf::from(destination);
  target.push(PODMANFILE_TARGET);

  let mut output = match fs::File::create(target) {
      Ok(o) => o,
      _ => return Err(ContainerError::new(FAILED_TO_CREATE_PODMAN_COMPOSE)),
  };

  match output.write_all(podmanfile.as_bytes()) {
      Ok(o) => Ok(o),
      _ => Err(ContainerError::new(FAILED_TO_WRITE_PODMAN_COMPOSE)),
  }	
}

pub fn write_podman_compose(
    destination: &path::PathBuf,
    config: &config::Config,
    podman_compose_filepath: &path::PathBuf,
) -> Result<(), ContainerError> {
    let contents = match fs::read_to_string(podman_compose_filepath) {
        Ok(c) => c,
        _ => return Err(ContainerError::new(PODMAN_COMPOSE_NOT_FOUND)),
    };
    
    let directory =  match config.directory.to_str() {
        Some(n) => n,
        _ => return Err(ContainerError::new(FAILED_TO_CONVERT_PODMAN_COMPOSE)),
    };
    
    let podman_compose = contents
        .replace("{host}", &config.host)
        .replace("{port}", &config.port.to_string())
        .replace("{directory}", directory);

		// write
		let mut target = path::PathBuf::from(destination);
    target.push(PODMAN_COMPOSE_TARGET);
    
    let mut output = match fs::File::create(target) {
        Ok(o) => o,
        _ => return Err(ContainerError::new(FAILED_TO_CREATE_PODMAN_COMPOSE)),
    };
    
    match output.write_all(podman_compose.as_bytes()) {
        Ok(o) => Ok(o),
        _ => Err(ContainerError::new(FAILED_TO_WRITE_PODMAN_COMPOSE)),
    }
}

