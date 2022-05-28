
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server, StatusCode};

use std::convert::Infallible;
use std::env;
use std::io;
use std::path::PathBuf;
use hyper::server::conn::AddrStream;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};


use tokio::fs::File;

mod config;
mod send_file;


#[tokio::main]
async fn main() {
    // get args
    let args = match env::args().nth(2) {
        Some(a) => a,
        None => return println!("no config params were found"),
    };

    let config_pathbuff = match config::get_pathbuff(&args) {
        Ok(pb) => pb,
        Err(e) => return println!("{:?}", e),
    };

    let config = match config::get_config(config_pathbuff) {
        Ok(c) => c,
        Err(e) => return println!("{:?}", e),
    };

    // // A `Service` is needed for every connection, so this
    // // creates one from our `hello_world` function.
    let make_svc = make_service_fn(|_| async {
        // service_fn converts our function into a `Service`
        Ok::<_, Infallible>(service_fn(|_req| async {
            send_file::send_file(&config,_req).await
        }))
    });

    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), config.port);
    let server = Server::bind(&addr).serve(make_svc);

    println!("config: {:?}", config);

    // // // Run this server for... forever!
    // // if let Err(e) = server.await {
    // //     eprintln!("server error: {}", e);
    // // }
}
