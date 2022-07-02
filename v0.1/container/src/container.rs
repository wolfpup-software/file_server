use std::env;
use std::path;
use std::fs;
use std::io::Write;
use std::fmt;
use config;


static CTNR_BASE_DIR: &str = "/usr/share/www";
static FILE_SERVER_JSON_TARGET: &str = "file-server.json";
static PODMAN_COMPOSE_TEMPLATE: &str = "podman-compose.yml.template";
static PODMAN_COMPOSE_TARGET: &str = "file-server.podman-compose.yml";
static PODMANFILE_TEMPLATE: &str = "podmanfile.template";
static PODMANFILE_TARGET: &str = "file-server.podmanfile";

static CONFIG_MOD_TARGET: &str = "../config";
static FILE_SERVER_MOD_TARGET: &str = "../file-server";

static FAILED_TO_CONVERT_CONFIG: &str = "failed to convert config into string";
static FAILED_TO_CREATE_CONFIG: &str = "failed to create config";
static FAILED_TO_WRITE_CONFIG: &str = "failed to write config to disk";

static PODMAN_COMPOSE_NOT_FOUND: &str = "podman-compose template was not found";
static FAILED_TO_CONVERT_PODMAN_COMPOSE: &str = "failed to convert podman-compose template to string";
static FAILED_TO_CREATE_PODMAN_COMPOSE: &str = "failed to create podman-compose";
static FAILED_TO_WRITE_PODMAN_COMPOSE: &str = "failed to wright podman-compose to disk";

static FAILED_TO_PARSE_REL_PATH: &str = "failed to create relative path";

static FAILED_TO_GET_MANIFEST_DIR: &str = "failed to get manifest dir";
static FAILED_TO_CREATE_CONFIG_MOD_PATH: &str = "failed to create config module path";
static FAILED_TO_CONVERT_CONFIG_MOD_PATH: &str = "failed to convert config module path";
static FAILED_TO_CREATE_FILE_SERVER_MOD_PATH: &str = "failed to create file server module path";
static FAILED_TO_CONVERT_FILE_SERVER_MOD_PATH: &str = "failed to convert file server module path";
static FAILED_TO_CREATE_JSON_PATH: &str = "failed to create config json path";
static FAILED_TO_CONVERT_JSON_PATH: &str = "failed to convert config json config path";

static PODMANFILE_NOT_FOUND: &str = "podmanfile template not found";
static FAILED_TO_CREATE_PODMANFILE: &str = "failed to create podmanfile";
static FAILED_TO_WRITE_PODMANFILE: &str = "failed to wright podmanfile to disk";


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
    	directory: base_dir,
    	port: config.port,
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
) -> Result<(), ContainerError> {
    let contents = match fs::read_to_string(PODMAN_COMPOSE_TEMPLATE) {
        Ok(c) => c,
        _ => return Err(ContainerError::new(PODMAN_COMPOSE_NOT_FOUND)),
    };

    let port = config.port.to_string();
    let directory = match config.directory.to_str() {
        Some(n) => n,
        _ => return Err(ContainerError::new(FAILED_TO_CONVERT_PODMAN_COMPOSE)),
    };

    let podman_compose = contents
        .replace("{port}", &port)
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

pub fn write_podmanfile(
    config: &config::Config,
    destination: &path::PathBuf,
) -> Result<(), ContainerError> {
    let repo_dir = match env::var("CARGO_MANIFEST_DIR") {
        Ok(p) => path::PathBuf::from(p),
        _ => return Err(ContainerError::new(FAILED_TO_GET_MANIFEST_DIR)),
    };

    let config_mod_dir = match repo_dir.join(CONFIG_MOD_TARGET).canonicalize() { 
        Ok(p) => p,
        _ => return Err(ContainerError::new(FAILED_TO_CREATE_CONFIG_MOD_PATH)),
    };
    let config_mod_dir_str = match config_mod_dir.to_str() { 
        Some(s) => s,
        _ => return Err(ContainerError::new(FAILED_TO_CONVERT_CONFIG_MOD_PATH)),
    };

    let file_server_mod_dir = match repo_dir.join(FILE_SERVER_MOD_TARGET).canonicalize() { 
        Ok(p) => p,
        _ => return Err(ContainerError::new(FAILED_TO_CREATE_FILE_SERVER_MOD_PATH)),
    };
    let file_server_mod_dir_str = match file_server_mod_dir.to_str() { 
        Some(s) => s,
        _ => return Err(ContainerError::new(FAILED_TO_CONVERT_FILE_SERVER_MOD_PATH)),
    };

    let config_json = match destination.join(FILE_SERVER_JSON_TARGET).canonicalize() {
        Ok(c) => c,
        _ => return Err(ContainerError::new(FAILED_TO_CREATE_JSON_PATH)),
    };
    let config_json_str = match config_json.to_str() { 
        Some(s) => s,
        _ => return Err(ContainerError::new(FAILED_TO_CONVERT_JSON_PATH)),
    };

    let contents = match fs::read_to_string(PODMANFILE_TEMPLATE) {
        Ok(c) => c,
        _ => return Err(ContainerError::new(PODMANFILE_NOT_FOUND)),
    };

    let port = config.port.to_string();
    let podman_compose = contents
        .replace("{port}", &port)
        .replace("{config_json_path}", config_json_str)
        .replace("{config_mod_dir}", config_mod_dir_str)
        .replace("{file_server_mod_dir}", file_server_mod_dir_str);

    let mut target = path::PathBuf::from(destination);
    target.push(PODMANFILE_TARGET);

    let mut output = match fs::File::create(target) {
        Ok(o) => o,
        _ => return Err(ContainerError::new(FAILED_TO_CREATE_PODMANFILE)),
    };

    match output.write_all(podman_compose.as_bytes()) {
        Ok(o) => Ok(o),
        _ => Err(ContainerError::new(FAILED_TO_WRITE_PODMANFILE)),
    }
}
