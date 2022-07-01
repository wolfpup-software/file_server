use std::env;
use std::path;
use std::fs;
use std::io::Write;
use std::fmt;
use config;


static FILE_SERVER_JSON: &str = "file-server.json";
static FAILED_TO_CONVERT_CONFIG: &str = "failed to convert config into a string";
static FAILED_TO_CREATE_CONFIG: &str = "failed to create podmanfile";
static FAILED_TO_WRITE_CONFIG: &str = "failed to write podmanfile";

static PODMAN_COMPOSE_NOT_FOUND: &str = "podman-compose file not found";
static PODMAN_COMPOSE_WRITE_FAILED: &str = "podman-compose file failed to write to disk";


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


pub fn get_dest_dir_from_args() -> Option<path::PathBuf> {
    match env::args().nth(2) {
        Some(c) => {
            let p = path::PathBuf::from(c);
            match p.is_dir() {
                true => Some(p),
                _ => None,
            }
        },
        _ => None,
    }
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
    target.push(FILE_SERVER_JSON);

    let mut output = match fs::File::create(target) {
        Ok(o) => o,
        _ => return Err(ContainerError::new(FAILED_TO_CREATE_CONFIG)),
    };

    match output.write_all(config.as_bytes()) {
        Ok(o) => Ok(()),
        _ => return Err(ContainerError::new(FAILED_TO_WRITE_CONFIG)),
    }
}

// container error
pub fn write_podman_compose(
    config: &config::Config,
    destination: &path::PathBuf,
) -> Result<(), ContainerError> {
    let contents = match fs::read_to_string("podman-compose.yml.template") {
        Ok(c) => c,
        _ => return Err(ContainerError::new(PODMAN_COMPOSE_NOT_FOUND)),
    };

    let port = config.port.to_string();
    let directory = match config.directory.to_str() {
        Some(n) => n,
        _ => return Err(ContainerError::new(PODMAN_COMPOSE_NOT_FOUND)),
    };

    let podman_compose = contents
        .replace("{port}", &port)
        .replace("{root_dir}", directory);

    let mut target = path::PathBuf::from(destination);
    target.push("file-server.podman-compose.yml");
    
    let mut output = match fs::File::create(target) {
        Ok(o) => o,
        _ => return Err(ContainerError::new(PODMAN_COMPOSE_WRITE_FAILED)),
    };
    match output.write_all(podman_compose.as_bytes()) {
        Ok(o) => Ok(()),
        _ => Err(ContainerError::new(PODMAN_COMPOSE_WRITE_FAILED)),
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
        _ => return Err(ContainerError::new(PODMAN_COMPOSE_NOT_FOUND)),
    };

    Ok(base_dir.join(stripped))
}

pub fn create_container_config(
    config: &config::Config,
    destination: &path::PathBuf,
) -> Result<config::Config, ContainerError> {
    let base_dir = path::PathBuf::from("/usr/share/www");

    let fp_403 = match pathbuff_to_container_pathbuff(
        &config.directory,
        &config.filepath_403,
        &base_dir,
    ) {
        Ok(fp) => fp,
        _ => return Err(ContainerError::new(PODMAN_COMPOSE_NOT_FOUND)),
    };

    let fp_404 = match pathbuff_to_container_pathbuff(
        &config.directory,
        &config.filepath_404,
        &base_dir,
    ) {
        Ok(fp) => fp,
        _ => return Err(ContainerError::new(PODMAN_COMPOSE_NOT_FOUND)),
    };

    let fp_500 = match pathbuff_to_container_pathbuff(
        &config.directory,
        &config.filepath_500,
        &base_dir,
    ) {
        Ok(fp) => fp,
        _ => return Err(ContainerError::new(PODMAN_COMPOSE_NOT_FOUND)),
    };

    Ok(config::Config {
    	directory: base_dir,
    	port: config.port,
    	filepath_403: fp_403,
    	filepath_404: fp_404,
    	filepath_500: fp_500,
    })
}

pub fn write_podmanfile(
    config: &config::Config,
    destination: &path::PathBuf,
) -> Result<(), ContainerError> {
    let repo_dir = match env::var("CARGO_MANIFEST_DIR") {
        Ok(p) => path::PathBuf::from(p),
        _ => return Err(ContainerError::new(PODMAN_COMPOSE_WRITE_FAILED)),
    };

    let config_dir = match repo_dir.join("../config").canonicalize() { 
        Ok(p) => p,
        _ => return Err(ContainerError::new(PODMAN_COMPOSE_WRITE_FAILED)),
    };
    let config_dir_str = match config_dir.to_str() { 
        Some(s) => s,
        _ => return Err(ContainerError::new(PODMAN_COMPOSE_WRITE_FAILED)),
    };

    let file_server_dir = match repo_dir.join("../file-server").canonicalize() { 
        Ok(p) => p,
        _ => return Err(ContainerError::new(PODMAN_COMPOSE_WRITE_FAILED)),
    };
    let file_server_dir_str = match file_server_dir.to_str() { 
        Some(s) => s,
        _ => return Err(ContainerError::new(PODMAN_COMPOSE_WRITE_FAILED)),
    };

    let config_json = match destination.join("file-server.json").canonicalize() {
        Ok(c) => c,
        _ => return Err(ContainerError::new(PODMAN_COMPOSE_WRITE_FAILED)),
    };
    let config_json_str = match config_json.to_str() { 
        Some(s) => s,
        _ => return Err(ContainerError::new(PODMAN_COMPOSE_WRITE_FAILED)),
    };

    let contents = match fs::read_to_string("podmanfile.template") {
        Ok(c) => c,
        _ => return Err(ContainerError::new(PODMAN_COMPOSE_NOT_FOUND)),
    };

    let port = format!("{}", config.port);
    let podman_compose = contents
        .replace("{port}", &port)
        .replace("{config_path}", config_json_str)
        .replace("{config_dir}", config_dir_str)
        .replace("{file_server_dir}", file_server_dir_str);

    let mut target = path::PathBuf::from(destination);
    target.push("file-server.podmanfile");

    let mut output = match fs::File::create(target) {
        Ok(o) => o,
        _ => return Err(ContainerError::new(PODMAN_COMPOSE_WRITE_FAILED)),
    };
    output.write_all(podman_compose.as_bytes());

    Ok(())
}
