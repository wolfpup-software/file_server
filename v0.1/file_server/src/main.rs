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
		None => return println!("argument error: no config params were found."),
	};

	let config = match config::Config::from_filepath(&args) {
		Ok(c) => c,
		Err(e) => return println!("configuration error: {}", e),
	};

	let address = format!("{}:{}", config.host, config.port);
	let listener = match TcpListener::bind(address).await {
		Ok(l) => l,
		_ => return println!("configuration error: unable to parse host."),
	};

	loop {
		let (stream, _) = match listener.accept().await {
			Ok(s) => s,
			_ => return println!("configuration error"),
		};
		
		let io = TokioIo::new(stream);
		let service = serve_file::Svc{
			directory: path::PathBuf::from(&config.directory),
		};
		
		tokio::task::spawn(async move {
			if let Err(err) = http1::Builder::new()
				.serve_connection(io, service)
				.await
			{
				// print or log error
				println!("Error serving connection: {:?}", err);
			}
		});
	}
}

