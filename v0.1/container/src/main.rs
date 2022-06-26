use std::env;
use std::path;
use std::fs;
use std::io::Write;
use config;

mod container;


fn main() {
    let config_filepath = match env::args().nth(1) {
        Some(fp) => fp,
        _ => return println!("args error: config filepath not found in args"),
    };

    let config = match config::get_config(&config_filepath) {
        Ok(c) => c,
        Err(e) => return println!("configuration error: {}", e),
    };

    let destination = match container::get_dest_dir_from_args() {
        Some(d) => d,
        _ => return println!("args error: destination directory not found."),
    };

    let container_config = match container::create_config(
        &config,
        &destination,
    ) {
        Ok(c) => c,
        _ => return println!("config error: failed to create container config")
    };

    container::write_config(
        &container_config,
        &destination,
    );

    container::write_podman_compose(
        &config,
        &destination,
    );

    container::create_podmanfile(
        &container_config,
        &destination,
    );
}