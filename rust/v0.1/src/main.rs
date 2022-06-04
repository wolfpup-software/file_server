use std::convert::Infallible;
use std::env;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use hyper::service::{make_service_fn, service_fn};
use hyper::{Server};

mod config;
mod send_file;


#[tokio::main]
async fn main() {
    // get json config
    let args = match env::args().nth(1) {
        Some(a) => a,
        None => return println!("no config params were found"),
    };

    let config_pathbuff = match config::get_pathbuff(&args) {
        Ok(pb) => pb,
        Err(e) => return println!("{}", e),
    };

    let config = match config::get_config(config_pathbuff) {
        Ok(c) => c,
        Err(e) => return println!("{}", e),
    };

    // create function for server (hyper::Server)
    let file_service = make_service_fn(|_| {
        let conf = config.clone();
        async {
            Ok::<_, Infallible>(service_fn(move |_req| {
                send_file::send_file(conf.clone(), _req)
            }))
        }
    });

    // create and run server
    let addr = SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        config.port,
    );

    let server = Server::bind(&addr).serve(file_service);
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
