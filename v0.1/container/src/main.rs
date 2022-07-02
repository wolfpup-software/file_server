use std::env;
use config;

mod container;


fn main() {
    let config_fp = match env::args().nth(1) {
        Some(fp) => fp,
        _ => return println!("args error: config filepath not found in args"),
    };

    let config = match config::get_config(&config_fp) {
        Ok(c) => c,
        Err(e) => return println!("configuration error: {}", e),
    };

    let destination = match container::get_dest_dir_from_args() {
        Some(d) => d,
        _ => return println!("args error: destination directory not found."),
    };

    let container_config = match container::create_container_config(
        &config,
    ) {
        Ok(c) => c,
        _ => return println!("config error: failed to create container config")
    };

    if let Err(e) = container::write_config(
        &container_config,
        &destination,
    ) {
        return println!("{}", e);
    };

    if let Err(e) = container::write_podman_compose(
        &config,
        &destination,
    ) {
        return println!("{}", e);
    };

    if let Err(e) = container::write_podmanfile(
        &container_config,
        &destination,
    ) {
        return println!("{}", e);
    };
}
