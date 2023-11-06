use std::env;
use std::path;

use hyper::server::conn::http1;
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

use config;

mod serve_file;


#[tokio::main]
async fn main() {
	let args = match env::args().nth(1) {
		Some(a) => path::PathBuf::from(a),
		None => return println!("argument error:\nno config params were found."),
	};

	let config = match config::Config::from_filepath(&args) {
		Ok(c) => c,
		Err(e) => return println!("configuration error:\n{}", e),
	};

	let address = format!("{}:{}", config.host, config.port);
	let listener = match TcpListener::bind(address).await {
		Ok(l) => l,
		Err(e) => return println!("configuration error:\n{}", e),
	};

	loop {
		let (stream, _) = match listener.accept().await {
			Ok(s) => s,
			Err(e) => return println!("configuration error:\n{}", e),
		};
		
		let io = TokioIo::new(stream);
		let service = serve_file::Svc{
			directory: path::PathBuf::from(&config.directory),
		};
		
		// print or log errors here
		tokio::task::spawn(async move {
			http1::Builder::new()
				.serve_connection(io, service)
				.await
		});
	}
}

