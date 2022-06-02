use std::convert::Infallible;
use std::env;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use hyper::service::{make_service_fn, service_fn};
use hyper::{Server};

mod config;
mod send_file;


#[tokio::main]
async fn main() {
    // get args
    println!("{:?}", env::args());
    let args = match env::args().nth(1) {
        Some(a) => a,
        None => return println!("no config params were found"),
    };

    let config_pathbuff = match config::get_pathbuff(&args) {
        Ok(pb) => pb,
        Err(e) => return println!("{:?}", e),
    };

    let config = match config::get_config_buffs(config_pathbuff) {
        Ok(c) => c,
        Err(e) => return println!("{:?}", e),
    };

    let top_scope_config = config.clone();

    let make_svc = make_service_fn(move |_| {
        let conf = config.clone();
        async move {
            Ok::<_, Infallible>(service_fn(move |_req| {
                send_file::send_file(conf.clone(), _req)
            }))
        }
    });

    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), top_scope_config.port);
    let server = Server::bind(&addr).serve(make_svc);

    println!("config: {:?}", top_scope_config);

    // Run this server for... forever!
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
