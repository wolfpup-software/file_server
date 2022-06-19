use std::convert::Infallible;
use std::env;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use hyper::service::{make_service_fn, service_fn};
use hyper::{Server, StatusCode};

mod config;
mod serve_file;


#[tokio::main]
async fn main() {
    // get json config
    let args = match env::args().nth(1) {
        Some(a) => a,
        None => return println!("argument error: no config params were found."),
    };

    let config = match config::get_config(&args) {
        Ok(c) => c,
        Err(e) => return println!("configuration error: {}", e),
    };

    // create function for server (hyper::Server)
    let file_service = make_service_fn(|_| {
        let conf = config.clone();
        
        async {
            Ok::<_, Infallible>(service_fn(move |_req| {
            	let dir = conf.directory.clone();
                let (status_code, pb) = match serve_file::get_pathbuff(&dir, _req) {
                	Ok(p) => match p.starts_with(&dir) {
                    		true => (StatusCode::OK, p),
                    		false => (StatusCode::FORBIDDEN, conf.filepath_403.clone()),
                    	},
                    	_ => (StatusCode::NOT_FOUND, conf.filepath_404.clone()),
                };

                serve_file::serve_file(status_code, pb, conf.filepath_500.clone())
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
        println!("server error: {}", e);
    }
}
