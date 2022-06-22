use std::env;
use std::path;

use config;

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
    
    let args2 = match env::args().nth(2) {
        Some(a) => a,
        None => return println!("2nd argument error: no destination was found."),
    };

    // get output dir
    let destination = path::PathBuf::from(args2);
    if !destination.is_dir() {
        return println!("2nd argument error: destination is not a dir")
    }

    
    // load file podmanfile.template
    
    // get formated String

    // save file to destination/podmanfile.template
    
    println!("{:?}", destination);
}
