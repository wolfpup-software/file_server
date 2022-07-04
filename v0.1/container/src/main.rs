use config;

mod container;


fn main() {
    let config_fp = match container::get_dest_filepath_from_args(2) {
        Some(fp) => fp,
        _ => return println!("args error: config filepath not found in args"),
    };

    println!("config_fp: {:?}", config_fp);

    let config = match config::Config::from_filepath(&config_fp) {
        Ok(c) => c,
        Err(e) => return println!("configuration error: {}", e),
    };

    let destination = match container::get_dest_dir_from_args(1) {
        Some(d) => d,
        _ => return println!("args error: destination directory not found."),
    };

    println!("config_fp: {:?}", destination);

    let podman_compose = match container::get_dest_filepath_from_args(3) {
        Some(d) => d,
        _ => return println!("args error: podman-compose template not found."),
    };

    println!("config_fp: {:?}", podman_compose);


    let container_config = match container::create_container_config(
        &config,
    ) {
        Ok(c) => c,
        _ => return println!("config error: failed to create container config")
    };

    println!("config_fp: {:?}", container_config);

    if let Err(e) = container::write_config(
        &container_config,
        &destination,
    ) {
        return println!("{}", e);
    };

    if let Err(e) = container::write_podman_compose(
        &config,
        &destination,
        &podman_compose,
    ) {
        return println!("{}", e);
    };
}
