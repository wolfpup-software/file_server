use std::env;
use std::path;

use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto::Builder;
use tokio::net::TcpListener;

mod config;
mod responses;


#[tokio::main]
async fn main() {
	let args = match env::args().nth(1) {
		Some(a) => path::PathBuf::from(a),
		None => return println!("argument error:\nconfig file not found."),
	};

	let config = match config::Config::from_filepath(&args) {
		Ok(c) => c,
		Err(e) => return println!("configuration error:\n{}", e),
	};

	let address = config.host.clone() + ":" + &config.port.to_string();
	let listener = match TcpListener::bind(address).await {
		Ok(l) => l,
		Err(e) => return println!("tcp listener error:\n{}", e),
	};

	// try not to panic during server loop
	loop {
		let (stream, _remote_address) = match listener.accept().await {
			Ok(s) => s,
			_ => {
				// log socket errors here
				continue;
			}
		};
		
		let io = TokioIo::new(stream);
		let service = responses::Svc{
			directory: config.directory.clone(),
		};
		
		tokio::task::spawn(async move {
			// log response errors here
			Builder::new(TokioExecutor::new())
				.serve_connection(io, service)
				.await
		});
	}
}

