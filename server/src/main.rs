mod service;

use config::get_service_requirements;
use config::AvailableEncodings;

use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto::Builder;
use std::env;
use std::path::PathBuf;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let config_path = match env::args().nth(1) {
        Some(dir) => PathBuf::from(dir),
        None => return println!("conf error: \nargs filepath missing from args"),
    };

    let config = match config::Config::try_from(&config_path).await {
        Ok(conf) => conf,
        Err(e) => return println!("conf error:\n{}", e),
    };

    let available_encodings = AvailableEncodings::new(&config.content_encodings);
    let service_requirements = get_service_requirements(&config);

    let listener = match TcpListener::bind(&config.host_and_port).await {
        Ok(lstnr) => lstnr,
        Err(e) => return println!("tcp listener error:\n{}", e),
    };

    loop {
        let (stream, ip_address) = match listener.accept().await {
            Ok(strm) => strm,
            Err(e) => return println!("tcp accept error:\n{}", e),
        };

        let io = TokioIo::new(stream);

        let service = service::Svc::new(
            &service_requirements,
            &available_encodings,
            &ip_address.to_string(),
        );

        tokio::task::spawn(async move {
            // log service errors here
            Builder::new(TokioExecutor::new())
                .serve_connection(io, service)
                .await
        });
    }
}
