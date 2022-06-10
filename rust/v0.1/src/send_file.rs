use std::convert::Infallible;
use std::io;
use std::path;

use hyper::{Body, Request, Response, StatusCode};
use hyper::header::CONTENT_TYPE;
use tokio::fs::File;
use tokio_util::codec::{BytesCodec, FramedRead};

use crate::config;


static INDEX: &str = "index";
static FWD_SLASH: &str = "/";
static ERROR: &[u8] = b"500 Internal Server Error";

// TEXT
const CSS_EXT: &str = "css";
const CSS: &str = "text/css";
const CSV: &str = "text/csv";
const CSV_EXT: &str = "csv";
const HTML_EXT: &str = "html";
const HTML: &str = "text/html; charset=utf-8";
const JS_EXT: &str = "js";
const JS: &str = "text/javascript";
const JSON_EXT: &str = "json";
const JSON: &str = "application/json";
const TEXT_EXT: &str = "txt";
const TEXT: &str = "text/plain; charset=utf-8";
const XML_EXT: &str = "xml";
const XML: &str = "application/xml";

// FONTS
const OTF_EXT: &str = "otf";
const OTF: &str = "font/otf";
const TTF_EXT: &str = "ttf";
const TTF: &str = "font/ttf";
const WOFF_EXT: &str = "woff";
const WOFF: &str = "font/woff";
const WOFF2_EXT: &str = "woff2";
const WOFF2: &str = "font/woff2";

// IMAGES
const BMP_EXT: &str = "bmp";
const BMP: &str = "image/bmp";
const GIF_EXT: &str = "gif";
const GIF: &str = "image/gif";
const ICO_EXT: &str = "ico";
const ICO: &str = "image/vnd.microsoft.icon";
const JPEG_EXT: &str = "jpeg";
const JPG_EXT: &str = "jpg";
const JPEG: &str = "image/jpeg";
const PDF_EXT: &str = "pdf";
const PDF: &str = "application/pdf";
const PNG_EXT: &str = "png";
const PNG: &str = "image/png";
const SVG_EXT: &str = "svg";
const SVG: &str = "image/svg+xml";
const TIFF_EXT: &str = "tiff";
const TIFF: &str = "image/tiff";
const WEBP_EXT: &str = "webp";
const WEBP: &str = "image/webp";

// AUDIO
const MIDI: &str = "audio/midi";
const AAC_EXT: &str = "aac";
const AAC: &str = "audio/aac";
const FLAC_EXT: &str = "flac";
const FLAC: &str = "audio/flac";
const MIDI_EXT: &str = "midi";
const MP3_EXT: &str = "mp3";
const MP3: &str = "audio/mpeg";
const OGGA_EXT: &str = "oga";
const OGGA: &str = "audio/ogg";
const WAV_EXT: &str = "wav";
const WAV: &str = "audio/wav";
const WEBA_EXT: &str = "weba";
const WEBA: &str = "audio/webm";

// VIDEO
const MP4_EXT: &str = "mp4";
const MP4: &str = "video/mp4";
const MPEG_EXT: &str = "mpeg";
const MPEG: &str = "video/mpeg";
const OGGV_EXT: &str = "ogv";
const OGGV: &str = "video/ogg";
const WEBM_EXT: &str = "webm";
const WEBM: &str = "video/webm";

// COMPRESSION
const GZIP_EXT: &str = "gz";
const GZIP: &str = "application/gzip";
const ZIP_EXT: &str = "zip";
const ZIP: &str = "application/zip";

// STREAMING
const TSV_EXT: &str = "ts";
const TSV: &str = "video/MP2T";
const M3U8_EXT: &str = "M3U8";
const M3U8: &str = "application/x-mpegURL";


fn get_content_type(request_path: &path::PathBuf) -> &str {
	let extension = match request_path.extension() {
		Some(ext) => {
			match ext.to_str() {
				Some(e) => e,
				None => TEXT_EXT,
			}
		},
		None => TEXT_EXT,
	};

	match extension {
		// many per page
		CSS_EXT => CSS,
		JS_EXT => JS,
		JSON_EXT => JSON,
		TSV_EXT => TSV,
		M3U8_EXT => M3U8,

		// several per page
		SVG_EXT => SVG,
		PNG_EXT => PNG,
		PDF_EXT => PDF,
		GIF_EXT => GIF,
		JPEG_EXT => JPEG,
		JPG_EXT => JPEG,
		TTF_EXT => TTF,
		WOFF_EXT => WOFF,
		WOFF2_EXT => WOFF2,
		OTF_EXT => OTF,

		// one per page
		HTML_EXT => HTML,
		GZIP_EXT => GZIP,
		ICO_EXT => ICO,

		// otherwise
		AAC_EXT => AAC,
		BMP_EXT => BMP,
		CSV_EXT => CSV,
		FLAC_EXT => FLAC,
		MIDI_EXT => MIDI,
		MP3_EXT => MP3,
		MP4_EXT => MP4,
		MPEG_EXT => MPEG,
		OGGA_EXT => OGGA,
		OGGV_EXT => OGGV,
		TEXT_EXT => TEXT,
		TIFF_EXT => TIFF,
		WAV_EXT => WAV,
		WEBA_EXT => WEBA,
		WEBM_EXT => WEBM,
		WEBP_EXT => WEBP,
		XML_EXT => XML,
		ZIP_EXT => ZIP,
		_ => TEXT,
	}
}

fn error_response() -> Response<Body> {
	Response::builder()
		.status(StatusCode::INTERNAL_SERVER_ERROR)
		.body(ERROR.into())
		.unwrap()
}

fn get_pathbuff(
	dir: &path::PathBuf,
	_req: &Request<Body>,
) -> Result<path::PathBuf, io::Error> {
    let mut path = path::PathBuf::from(dir);
    let uri_path = _req.uri().path();
    let stripped_path = match uri_path.strip_prefix(FWD_SLASH) {
        Some(p) => p,
        None => uri_path,
    };

    path.push(stripped_path);
    if path.is_dir() {
        path.push(INDEX);
        path.set_extension(HTML_EXT);
    }

    path.canonicalize()?;
    Ok(path)
}

async fn serve_file(
	request_path: path::PathBuf,
	status_code: StatusCode,
) -> Result<Response<Body>, std::io::Error> {
	match File::open(&request_path).await {
		Ok(file) => {
			let content_type = get_content_type(&request_path);
			let stream = FramedRead::new(file, BytesCodec::new());
			let body = Body::wrap_stream(stream);
			let response = Response::builder()
				.status(status_code)
				.header(CONTENT_TYPE, content_type)
				.body(body)
				.unwrap();
			
			Ok(response)
		},
		Err(e) => Err(e),
	}
}

pub async fn send_file(
	config: config::Config,
	_req: Request<Body>,
) -> Result<Response<Body>, Infallible> {
	// assume request is inherently unauthorized
	let mut status_code = StatusCode::FORBIDDEN;
	let mut pb = config.filepath_403;
    
	match get_pathbuff(&config.dir, &_req) {
		Ok(p) => {
			if p.starts_with(&config.dir) {
					status_code = StatusCode::OK;
					pb = p;
			}
		},
		Err(_) => {
			status_code = StatusCode::NOT_FOUND;
			pb = config.filepath_404;
		},
	}

	// attempt to serve default responses
	if let Ok(response) = serve_file(pb, status_code).await {
		return Ok(response);
	};

	if let Ok(response) = serve_file(
		config.filepath_500,
		StatusCode::INTERNAL_SERVER_ERROR,
	).await {
  		return Ok(response);
  };

	// last ditch error
	Ok(error_response())
}
