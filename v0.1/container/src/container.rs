use std::env;
use std::path;
use std::fs;
use std::io::Write;
use std::fmt;
use config;

static CNTR_HOST: &str = "0.0.0.0";
static CTNR_BASE_DIR: &str = "/usr/share/www";
static FILE_SERVER_JSON_TARGET: &str = "file-server.json";
static PODMAN_COMPOSE_TARGET: &str = "file-server.podman-compose.yml";

static FAILED_TO_CONVERT_CONFIG: &str = "failed to convert config into string";
static FAILED_TO_CREATE_CONFIG: &str = "failed to create config";
static FAILED_TO_WRITE_CONFIG: &str = "failed to write config to disk";

static PODMAN_COMPOSE_NOT_FOUND: &str = "podman-compose template was not found";
static FAILED_TO_CONVERT_PODMAN_COMPOSE: &str = "failed to convert podman-compose template to string";
static FAILED_TO_CREATE_PODMAN_COMPOSE: &str = "failed to create podman-compose";
static FAILED_TO_WRITE_PODMAN_COMPOSE: &str = "failed to wright podman-compose to disk";

static FAILED_TO_PARSE_REL_PATH: &str = "failed to create relative path";


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


pub fn get_dest_dir_from_args(index: usize) -> Option<path::PathBuf> {
    // get destination
    match env::args().nth(index) {
        Some(c) => {
            let p = path::PathBuf::from(c);
            println!("get_dest_dir: {:?}", p);
            match p.is_dir() {
                true => Some(p),
                _ => None,
            }
        },
        _ => None,
    }
}

pub fn get_dest_filepath_from_args(index: usize) -> Option<path::PathBuf> {
    // get destination
    match env::args().nth(index) {
        Some(c) => {
            let p = path::PathBuf::from(c);
            println!("get_dest_dir: {:?}", p);
            match p.is_file() {
                true => Some(p),
                _ => None,
            }
        },
        _ => None,
    }
}


fn pathbuff_to_container_pathbuff(
    directory: &path::PathBuf,
    filepath: &path::PathBuf,
    base_dir: &path::PathBuf,
) -> Result<path::PathBuf, ContainerError> {
    let fp = path::PathBuf::from(filepath);
    let stripped = match fp.strip_prefix(directory) {
        Ok(f) => f,
        _ => return Err(ContainerError::new(FAILED_TO_PARSE_REL_PATH)),
    };

    Ok(base_dir.join(stripped))
}

pub fn create_container_config(
    config: &config::Config,
) -> Result<config::Config, ContainerError> {
    let base_dir = path::PathBuf::from(CTNR_BASE_DIR);

    let fp_403 = match pathbuff_to_container_pathbuff(
        &config.directory,
        &config.filepath_403,
        &base_dir,
    ) {
        Ok(fp) => fp,
        Err(e) => return Err(e),
    };

    let fp_404 = match pathbuff_to_container_pathbuff(
        &config.directory,
        &config.filepath_404,
        &base_dir,
    ) {
        Ok(fp) => fp,
        Err(e) => return Err(e),
    };

    let fp_500 = match pathbuff_to_container_pathbuff(
        &config.directory,
        &config.filepath_500,
        &base_dir,
    ) {
        Ok(fp) => fp,
        Err(e) => return Err(e),
    };

    Ok(config::Config {
        host: CNTR_HOST.to_string(),
    	port: 3000,
    	directory: base_dir,
    	filepath_403: fp_403,
    	filepath_404: fp_404,
    	filepath_500: fp_500,
    })
}

pub fn write_config(
    config: &config::Config,
    destination: &path::PathBuf,
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

// container error
pub fn write_podman_compose(
    config: &config::Config,
    destination: &path::PathBuf,
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
        .replace("{root_dir}", directory);

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
