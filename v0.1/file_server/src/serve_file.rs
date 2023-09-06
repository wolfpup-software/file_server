use std::convert::Infallible;
use std::io;
use std::path;

use hyper::{Body, Request, Response, StatusCode};
use hyper::header::{HeaderValue, CONTENT_TYPE};
use tokio_util::codec::{BytesCodec, FramedRead};
use tokio::fs::File;


const INDEX: &str = "index";
const FWD_SLASH: &str = "/";
const ERROR: &str = "500 Internal Server Error";

// TEXT
const CSS_EXT: &str = "css";
const CSS: &str = "text/css; charset=utf-8";
const CSV_EXT: &str = "csv";
const CSV: &str = "text/csv; charset=utf-8";
const HTML_EXT: &str = "html";
const HTML: &str = "text/html; charset=utf-8";
const JS_EXT: &str = "js";
const JS: &str = "text/javascript; charset=utf-8";
const JSON_EXT: &str = "json";
const JSON: &str = "application/json; charset=utf-8";
const TEXT_EXT: &str = "txt";
const TEXT: &str = "text/plain; charset=utf-8";
const XML_EXT: &str = "xml";
const XML: &str = "application/xml; charset=utf-8";

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
const AAC_EXT: &str = "aac";
const AAC: &str = "audio/aac";
const FLAC_EXT: &str = "flac";
const FLAC: &str = "audio/flac";
const MIDI_EXT: &str = "midi";
const MIDI: &str = "audio/midi";
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
const M3U8_EXT: &str = "M3U8";
const M3U8: &str = "application/x-mpegURL";
const TSV_EXT: &str = "ts";
const TSV: &str = "video/MP2T";


fn response_500() -> Response<Body> {
	let mut response: Response<Body> = Response::new(ERROR.into());
	*response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
	response.headers_mut().insert(CONTENT_TYPE, HeaderValue::from_static(HTML));

	response
}

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
		CSS_EXT => CSS,
		JS_EXT => JS,
		JSON_EXT => JSON,
		TSV_EXT => TSV,
		M3U8_EXT => M3U8,
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
		HTML_EXT => HTML,
		GZIP_EXT => GZIP,
		ICO_EXT => ICO,
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

pub fn get_pathbuff_from_request(
	dir: &path::PathBuf,
	_req: Request<Body>,
) -> Result<path::PathBuf, io::Error> {
    let uri = _req.uri().path();
    let strip_uri = match uri.strip_prefix(FWD_SLASH) {
        Some(p) => p,
        None => uri,
    };

    let mut path = dir.join(strip_uri);
    if path.is_dir() {
        path.push(INDEX);
        path.set_extension(HTML_EXT);
    }

    path.canonicalize()
}

async fn build_response(
	status_code: StatusCode,
	request_path: path::PathBuf,
	file: File,
) -> Result<Response<Body>, hyper::http::Error> {
	let content_type = get_content_type(&request_path);
	let stream = FramedRead::new(file, BytesCodec::new());
	let body = Body::wrap_stream(stream);

	Response::builder()
		.status(status_code)
		.header(CONTENT_TYPE, content_type)
		.body(body)
}

pub async fn serve_path(
	status_code: StatusCode,
	pb: path::PathBuf,
	pb_500: path::PathBuf,
) -> Result<Response<Body>, Infallible> {
	// attempt to serve file
	if let Ok(file) = File::open(&pb).await {
		if let Ok(response) = build_response(status_code, pb, file).await {
			return Ok(response);
		}
	};

	// custom 500
	if let Ok(file) = File::open(&pb_500).await {
		if let Ok(response) = build_response(
			StatusCode::INTERNAL_SERVER_ERROR,
			pb_500,
			file,
		).await {
			return Ok(response);
		}
	};

	// oh no 500
	Ok(response_500())
}