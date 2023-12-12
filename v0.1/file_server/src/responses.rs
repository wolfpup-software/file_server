use std::future::Future;
use std::io;
use std::path;
use std::pin::Pin;


use futures_util::TryStreamExt;
use http_body_util::{combinators::BoxBody, BodyExt, Full, StreamBody};
use hyper::{Request, Response, StatusCode};
use hyper::body::{Frame, Incoming as IncomingBody};
use hyper::header::{HeaderValue, CONTENT_TYPE};
use hyper::service::Service;
use tokio::fs::File;
use tokio_util::io::ReaderStream;


const FWD_SLASH: &str = "/";
const INDEX: &str = "index";
const NOT_FOUND: &str = "404 not found";
const INTERNAL_SERVER_ERROR: &str = "500 internal server error";

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
const OCTET_STREAM: &str = "application/octet-stream";

// BINARY
const WASM_EXT: &str = "wasm";
const WASM: &str = "application/wasm";


pub struct Svc {
	pub directory: path::PathBuf,
}

impl Service<Request<IncomingBody>> for Svc {
	type Response = Response<BoxBody<bytes::Bytes, std::io::Error>>;
	type Error = hyper::http::Error;
	type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

	fn call(&self, req: Request<IncomingBody>) -> Self::Future {
		let path = match get_pathbuff_from_request(
			&self.directory,
			&req,
		) {
			Ok(p) => p,
			Err(_err) =>  {
				return Box::pin(async {response_404()});
			},
		};
		
		if !path.starts_with(&self.directory) {
			return Box::pin(async {response_404()});
		}
		
		Box::pin(async {
		  build_response(path).await
		})
	}
}

fn get_pathbuff_from_request(
	dir: &path::PathBuf,
	req: &Request<IncomingBody>,
) -> Result<path::PathBuf, io::Error> {
	let uri = req.uri().path();
	let strip_uri = match uri.strip_prefix(FWD_SLASH) {
		Some(p) => p,
		None => uri,
	};

	// just join the directory again, use a match
	let mut path = dir.join(strip_uri);
	if path.is_dir() {
		path.push(INDEX);
		path.set_extension(HTML_EXT);
	}

	path.canonicalize()
}

fn get_content_type(request_path: &path::PathBuf) -> &str {
	let extension = match request_path.extension() {
		Some(ext) => {
			match ext.to_str() {
				Some(e) => e,
				_ => TEXT,
			}
		},
		// text files with no extension
		_ => TEXT, 
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
		WASM_EXT => WASM,
		WAV_EXT => WAV,
		WEBA_EXT => WEBA,
		WEBM_EXT => WEBM,
		WEBP_EXT => WEBP,
		XML_EXT => XML,
		ZIP_EXT => ZIP,
		_ => OCTET_STREAM,
	}
}

fn response_404() -> Result<Response<BoxBody<bytes::Bytes, std::io::Error>>, hyper::http::Error> {
  Response::builder()
		.status(StatusCode::NOT_FOUND)
		.header(CONTENT_TYPE, HeaderValue::from_static(HTML))
		// painful, more ergonomic?
		.body(Full::new(NOT_FOUND.into()).map_err(|e| match e {}).boxed())
}

fn response_500() -> Result<Response<BoxBody<bytes::Bytes, std::io::Error>>, hyper::http::Error> {
	Response::builder()
		.status(StatusCode::INTERNAL_SERVER_ERROR)
		.header(CONTENT_TYPE, HeaderValue::from_static(HTML))
		.body(Full::new(INTERNAL_SERVER_ERROR.into()).map_err(|e| match e {}).boxed())
}

async fn build_response(
	path: path::PathBuf,
) -> Result<Response<BoxBody<bytes::Bytes, std::io::Error>>, hyper::http::Error> {
		match File::open(&path).await {
			Ok(file) => {
				// from https://github.com/hyperium/hyper/blob/master/examples/send_file.rs
				let content_type = get_content_type(&path);
				let reader_stream = ReaderStream::new(file);
				let stream_body = StreamBody::new(reader_stream.map_ok(Frame::data));
				let boxed_body = stream_body.boxed();
				
				Response::builder()
					.status(StatusCode::OK)
					.header(CONTENT_TYPE, content_type)
					.body(boxed_body)
			},
			_ => response_500()
		}
}

