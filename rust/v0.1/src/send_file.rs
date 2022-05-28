use std::path;
use std::fmt;
use std::io;
use std::convert::Infallible;

use hyper::{Body, Request, Response, Server, StatusCode};
use tokio::fs::File;
use tokio_util::codec::{BytesCodec, FramedRead};

static INDEX: &str = "index";
static HTML: &str = "html";

static ERROR: &[u8] = b"500 Internal server error";

use crate::config;

fn get_pathbuff(dir: &path::PathBuf, _req: &Request<Body>) -> Result<path::PathBuf, io::Error> {
    let mut path = path::PathBuf::from(dir);
    path.push(_req.uri().path());
    path.canonicalize()?;

    if path.is_dir() {
        path.push(INDEX);
        path.set_extension(HTML);
    }

    Ok(path)
}

// can be cretaed and cached
fn error_response() -> Response<Body> {
    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(ERROR.into())
        .unwrap()
}

fn valid_path(base_dir: &path::PathBuf, request_path: &path::PathBuf) -> bool {
    !request_path.starts_with(base_dir)
}

// add file sieve
async fn serve_file(request_path: &path::PathBuf, status_code: StatusCode) -> Option<Response<Body>> {
    if let Ok(file) = File::open(request_path).await {
        let stream = FramedRead::new(file, BytesCodec::new());
        let body = Body::wrap_stream(stream);
        let response = Response::builder()
            .status(status_code)
            .body(body)
            .unwrap();
        
        // response new, set type
        return Some(response);
    }

    None
}

pub async fn send_file(config: &'static config::Config, _req: Request<Body>) -> Result<Response<Body>, Infallible> {
    // let mut status_code;
    // match get_pathbuff(&config.dir, &_req) {
    //     Ok(pb) => {
    //         if valid_path(&config.dir, &pb) {
    //             status_code = StatusCode::BAD_REQUEST;
    //         }

    //         if let Some(response) = serve_file(&pb, StatusCode::OK).await {
    //             return Ok(response);
    //         };

    //         status_code = StatusCode::NOT_FOUND;
    //     },
    //     Err(_) => {
    //         status_code = StatusCode::NOT_FOUND;
    //     },
    // }
    
    // if status_code == StatusCode::BAD_REQUEST && valid_path(&config.dir, &config.filepath_400) {
    //     if let Some(response) = serve_file(&config.filepath_400, StatusCode::BAD_REQUEST).await {
    //         return Ok(response);
    //     };
    // }

    // if status_code == StatusCode::NOT_FOUND && valid_path(&config.dir, &config.filepath_404) {
    //     if let Some(response) = serve_file(&config.filepath_404, StatusCode::NOT_FOUND).await {
    //         return Ok(response);
    //     };
    // }

    Ok(error_response())
}

