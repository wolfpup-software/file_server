use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto::Builder;
use std::env;
use std::path::PathBuf;
use tokio::net::TcpListener;

mod config;
mod service;

#[tokio::main]
async fn main() {
    let config_path = match env::args().nth(1) {
        Some(dir) => PathBuf::from(dir),
        None => return println!("conf error: \nargs filepath missing from args"),
    };

    let conf = match config::Config::try_from(&config_path).await {
        Ok(conf) => conf,
        Err(e) => return println!("conf error:\n{}", e),
    };

    let listener = match TcpListener::bind(conf.host_and_port).await {
        Ok(lstnr) => lstnr,
        Err(e) => return println!("tcp listener error:\n{}", e),
    };

    let svc = service::Svc::new(conf.directory, conf.content_encodings, conf.filepath_404);

    loop {
        let (stream, _remote_address) = match listener.accept().await {
            Ok(strm) => strm,
            Err(e) => return println!("tcp accept error:\n{}", e),
        };

        let io = TokioIo::new(stream);
        let svc = svc.clone();

        tokio::task::spawn(async move {
            // log service errors here
            Builder::new(TokioExecutor::new())
                .serve_connection(io, svc)
                .await
        });
    }
}
