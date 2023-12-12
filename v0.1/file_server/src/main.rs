use std::env;
use std::path;

use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto::Builder;
use tokio::net::TcpListener;

use config;

mod responses;


#[tokio::main]
async fn main() {
	let args = match env::args().nth(1) {
		Some(a) => path::PathBuf::from(a),
		None => return println!("argument error:\nconfig file was not found."),
	};

	let config = match config::Config::from_filepath(&args) {
		Ok(c) => c,
		Err(e) => return println!("configuration error:\n{}", e),
	};

	let address = format!("{}:{}", config.host, config.port);
	let listener = match TcpListener::bind(address).await {
		Ok(l) => l,
		Err(e) => return println!("tcp listener error:\n{}", e),
	};

	loop {
		let (stream, _) = match listener.accept().await {
			Ok(s) => s,
			Err(e) => return println!("socket error:\n{}", e),
		};
		
		let io = TokioIo::new(stream);
		let service = responses::Svc{
			directory: config.directory.clone(),
		};
		
		// print or log errors here
		tokio::task::spawn(async move {
			Builder::new(TokioExecutor::new())
				.serve_connection(io, service)
				.await
		});
	}
}

