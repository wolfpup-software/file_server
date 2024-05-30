use std::env;

use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto::Builder;
use tokio::net::TcpListener;

mod responses;
mod service;

#[tokio::main]
async fn main() {
    let address = match env::args().nth(1) {
        Some(addr) => addr,
        None => return println!("argument error:\nconfig file not found."),
    };

    let listener = match TcpListener::bind(address).await {
        Ok(lstnr) => lstnr,
        Err(e) => return println!("tcp listener error:\n{}", e),
    };

    let cwd = match env::current_dir() {
        Ok(addr) => addr,
        Err(e) => return println!("directory error:\n{}", e),
    };

    loop {
        let (stream, _remote_address) = match listener.accept().await {
            Ok(strm) => strm,
            _ => {
                // log socket errors here
                continue;
            }
        };

        let io = TokioIo::new(stream);
        let service = service::Svc {
            directory: cwd.clone(),
        };

        tokio::task::spawn(async move {
            // log service errors here
            Builder::new(TokioExecutor::new())
                .serve_connection(io, service)
                .await
        });
    }
}
