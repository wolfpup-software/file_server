use std::convert::Infallible;
use std::net::SocketAddr;

use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

async fn hello(_: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    Ok(Response::new(Full::new(Bytes::from("Hello, World!"))))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    // We create a TcpListener and bind it to 127.0.0.1:3000
    let listener = TcpListener::bind(addr).await?;

    // We start a loop to continuously accept incoming connections
    loop {
        let (stream, _) = listener.accept().await?;

        // Use an adapter to access something implementing `tokio::io` traits as if they implement
        // `hyper::rt` IO traits.
        let io = TokioIo::new(stream);

        // Spawn a tokio task to serve multiple connections concurrently
        tokio::task::spawn(async move {
            // Finally, we bind the incoming connection to our `hello` service
            if let Err(err) = http1::Builder::new()
                // `service_fn` converts our function in a `Service`
                .serve_connection(io, service_fn(hello))
                .await
            {
                println!("Error serving connection: {:?}", err);
            }
        });
    }
}

/*

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
*/

