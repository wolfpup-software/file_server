use std::env;

use config;

fn main() {
    let args = match env::args().nth(1) {
        Some(a) => a,
        None => return println!("argument error: no config params were found."),
    };

    let config = match config::get_config(&args) {
        Ok(c) => c,
        Err(e) => return println!("configuration error: {}", e),
    };
    
    println!("{:?}", config);
}
