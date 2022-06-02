use std::convert::Infallible;
use std::fmt;
use std::io;
use std::path;

use hyper::{Body, Request, Response, StatusCode};
use tokio::fs::File;
use tokio_util::codec::{BytesCodec, FramedRead};

use crate::config;

static INDEX: &str = "index";
static HTML: &str = "html";
static FWD_SLASH: &str = "/";
static ERROR: &[u8] = b"500 Internal server error";


#[derive(Debug)]
pub struct SendFileError {
    message: String,
}

impl SendFileError {
    pub fn new(message: String) -> SendFileError {
        SendFileError { message }
    }
    
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

fn get_pathbuff(dir: &path::PathBuf, _req: &Request<Body>) -> Result<path::PathBuf, io::Error> {
    let mut path = path::PathBuf::from(dir);
    println!("dir {:?}", path);

    // path.push(_req.uri().path().strip_prefix("/"));
    let uri_path = _req.uri().path();
    let stripped_path = match uri_path.strip_prefix(FWD_SLASH) {
        Some(p) => p,
        None => uri_path,
    };

    path.push(stripped_path);
    println!("path {:?}", path);
    path.canonicalize()?;
    println!("canon path: {:?}", path);

    if path.is_dir() {
        path.push(INDEX);
        path.set_extension(HTML);
    }

    println!("final path: {:?}", path);
    println!("final path exists: {:?}", path.exists());
    
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
        // get file type and add extension to request
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

pub async fn send_file(config: config::ConfigBuffs, _req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let mut status_code;
    match get_pathbuff(&config.dir, &_req) {
        Ok(pb) => {
            if !valid_path(&config.dir, &pb) {
                println!("not valid path!");
                status_code = StatusCode::BAD_REQUEST;
            }

            if let Some(response) = serve_file(&pb, StatusCode::OK).await {
                println!("file was ok!");
                return Ok(response);
            };

            status_code = StatusCode::NOT_FOUND;
        },
        Err(_) => {
            status_code = StatusCode::NOT_FOUND;
        },
    }
    
    println!("final path: {:?}", status_code);

    if status_code == StatusCode::BAD_REQUEST && valid_path(&config.dir, &config.filepath_400) {
        println!("serving 400!");
        if let Some(response) = serve_file(&config.filepath_400, StatusCode::BAD_REQUEST).await {
            return Ok(response);
        };
    }

    if status_code == StatusCode::NOT_FOUND && valid_path(&config.dir, &config.filepath_404) {
        println!("serving 404!");
        if let Some(response) = serve_file(&config.filepath_404, StatusCode::NOT_FOUND).await {
            return Ok(response);
        };
    }

    println!("didn't find nothing!!");
    Ok(error_response())
}
