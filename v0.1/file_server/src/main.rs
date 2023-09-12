use std::convert::Infallible;
use std::env;
use std::net;
use std::path;

use hyper::{Server, StatusCode};
use hyper::service::{make_service_fn, service_fn};

use config;

mod serve_file;


#[tokio::main]
async fn main() {
    // get config
    let args = match env::args().nth(1) {
        Some(a) => path::PathBuf::from(a),
        None => return println!("argument error: no config params were found."),
    };

    let config = match config::Config::from_filepath(&args) {
        Ok(c) => c,
        Err(e) => return println!("configuration error: {}", e),
    };

    let host = match config.host.parse() {
        Ok(h) => h,
        _ => return println!("configuration error: unable to parse host."),
    };

    let addr = net::SocketAddr::new(host, config.port);

    // create function for server (hyper::Server)
    // look up best fit for making service function
    // make this closure it's own function
    //
    // do we need 
    let file_service = make_service_fn(|_| {
    		// only need 500 and 404 path
        let conf = config::ServiceConfig::from_config(&config);
        
        async {
            Ok::<_, Infallible>(service_fn(move |_req| {
            		// get path buff
            		// if error return 404
            		// otherwise get file
            		// then serve file
                let (status_code, pb) = match serve_file::get_pathbuff_from_request(&conf.directory, _req) {
                	Ok(p) => match p.starts_with(&conf.directory) {
                		true => (StatusCode::OK, p),
                		_ => (StatusCode::NOT_FOUND, conf.filepath_404.clone()),
                	},
                	_ => (StatusCode::NOT_FOUND, conf.filepath_404.clone()),
                };

                serve_file::serve_path(status_code, pb, conf.filepath_500.clone())
            }))
        }
    });

    // run server
    if let Err(e) = Server::bind(&addr).serve(file_service).await {
        println!("server error: {}", e);
    }
}

async fn file_response(req: Request<hyper::body::Incoming>) {

}
