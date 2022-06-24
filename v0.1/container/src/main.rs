use std::env;
use std::path;
use std::fs;
use std::io::Write;
use config;

mod templates;


fn main() {
    let args = match env::args().nth(1) {
        Some(a) => a,
        None => return println!("1st argument error: no config params were found."),
    };

    let config = match config::get_config(&args) {
        Ok(c) => c,
        Err(e) => return println!("configuration error: {}", e),
    };

    let curr_dir = match env::current_dir() {
        Ok(pb) => pb,
        _ => return println!("current_dir error: no current_dir")
    };
    
    let destination = match env::args().nth(2) {
        Some(a) => path::PathBuf::from(a),
        None => return println!("2nd argument error: no destination was found."),
    };
    if !destination.is_dir() {
        return println!("2nd argument error: destination is not a dir")
    };

    let contents = match fs::read_to_string("podmanfile.template") {
        Ok(c) => c,
        _ => return println!("template error: no podmanfile found"),
    };
    let podmanfile = contents.replace("{port}", "3000");

    println!("{:?}", podmanfile);

    // output podman file to directory
    let mut output = match fs::File::create("file-server.podmanfile") {
        Ok(o) => o,
        _ => return println!("template error: could not write podmanfile template.")
    };
    output.write_all(podmanfile.as_bytes());

    let contents1 = match fs::read_to_string("podman-compose.yml.template") {
        Ok(c) => c,
        _ => return println!("template error: no podmanfile-compose found"),
    };

    let podman_file = contents1
        .replace("{port}", "3000")
        .replace("{directory}", "./hello!");
    
    println!("{:?}", podman_file);


}
