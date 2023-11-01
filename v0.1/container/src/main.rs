use config;

mod container;


fn main() {
	let destination = match container::get_pathbuff_from_args(1) {
		Some(d) => d,
		_ => return println!("args error: destination directory not found."),
	};
	
	// write container config
	let container_config = match container::create_container_config() {
		Ok(c) => c,
		_ => return println!("config error: failed to create container config")
	};
	
	if let Err(e) = container::write_config(
		&destination,
		&container_config,
	) {
		return println!("error writing config: {}", e);
	};

	// config
	let config_fp = match container::get_pathbuff_from_args(2) {
		Some(fp) => fp,
		_ => return println!("args error: config filepath not found in args"),
	};
	let config = match config::Config::from_filepath(&config_fp) {
		Ok(c) => c,
		Err(e) => return println!("configuration error: {}", e),
	};
	
	// write podmanfile
	let podmanfile = match container::get_pathbuff_from_args(3) {
		Some(d) => d,
		_ => return println!("args error: podman-compose template not found."),
	};
	
	if let Err(e) = container::write_podmanfile(
		&destination,
		&podmanfile,
	) {
		return println!("error writing podmanfile: {}", e);
	};

	// write podman compose
	let podman_compose = match container::get_pathbuff_from_args(4) {
		Some(d) => d,
		_ => return println!("args error: podman-compose template not found."),
	};
	
	if let Err(e) = container::write_podman_compose(
		&destination,
		&config,
		&podman_compose,
	) {
		return println!("error writing podman-compose: {}", e);
	};
}

