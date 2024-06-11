use std::path;

pub const HTML: &str = "text/html; charset=utf-8";
// TEXT
const CSS_EXT: &str = "css";
const CSS: &str = "text/css; charset=utf-8";
const CSV_EXT: &str = "csv";
const CSV: &str = "text/csv; charset=utf-8";
const HTML_EXT: &str = "html";
const JS_EXT: &str = "js";
const JS: &str = "text/javascript; charset=utf-8";
const JSON_EXT: &str = "json";
const JSON: &str = "application/json; charset=utf-8";
const TEXT_EXT: &str = "txt";
const TEXT: &str = "text/plain; charset=utf-8";
const XML_EXT: &str = "xml";
const XML: &str = "application/xml; charset=utf-8";
const WEB_MANIFEST_JSON_EXT: &str = "webmanifest";
const WEB_MANIFEST_JSON: &str = "application/manifest+json";

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
const ZSTD_EXT: &str = "zstd";
const BR_EXT: &str = "br";
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

// A file with no extention is still a textfile.
// Directories would be transformed into a index.html path.
pub fn get_content_type(path: &path::PathBuf) -> &str {
    let alt_path = get_alt_path(path);

    let alt_extension = match alt_path.extension() {
        Some(ext) => ext,
        _ => return TEXT,
    };

    let ext_str = match alt_extension.to_str() {
        Some(e) => e,
        _ => return TEXT,
    };

    match ext_str {
        AAC_EXT => AAC,
        BMP_EXT => BMP,
        CSS_EXT => CSS,
        CSV_EXT => CSV,
        FLAC_EXT => FLAC,
        GIF_EXT => GIF,
        GZIP_EXT => GZIP,
        HTML_EXT => HTML,
        ICO_EXT => ICO,
        JPEG_EXT => JPEG,
        JPG_EXT => JPEG,
        JS_EXT => JS,
        JSON_EXT => JSON,
        WEB_MANIFEST_JSON_EXT => WEB_MANIFEST_JSON,
        M3U8_EXT => M3U8,
        MIDI_EXT => MIDI,
        MP3_EXT => MP3,
        MP4_EXT => MP4,
        MPEG_EXT => MPEG,
        OGGA_EXT => OGGA,
        OGGV_EXT => OGGV,
        OTF_EXT => OTF,
        PDF_EXT => PDF,
        PNG_EXT => PNG,
        SVG_EXT => SVG,
        TEXT_EXT => TEXT,
        TIFF_EXT => TIFF,
        TSV_EXT => TSV,
        TTF_EXT => TTF,
        WASM_EXT => WASM,
        WAV_EXT => WAV,
        WEBA_EXT => WEBA,
        WEBM_EXT => WEBM,
        WEBP_EXT => WEBP,
        WOFF2_EXT => WOFF2,
        WOFF_EXT => WOFF,
        XML_EXT => XML,
        ZIP_EXT => ZIP,
        _ => OCTET_STREAM,
    }
}

pub fn get_content_encoding(path: &path::PathBuf) -> Option<&str> {
    let extension = match path.extension() {
        Some(ext) => ext,
        _ => return None,
    };

    let ext_str = match extension.to_str() {
        Some(e) => e,
        _ => return None,
    };

    match ext_str {
        GZIP_EXT => Some(GZIP_EXT),
        BR_EXT => Some(BR_EXT),
        ZSTD_EXT => Some(ZSTD_EXT),
        _ => None,
    }
}

fn compressed_ext(ext_str: &str) -> bool {
    match ext_str {
        GZIP_EXT => true,
        ZSTD_EXT => true,
        BR_EXT => true,
        _ => false,
    }
}

fn get_alt_path(path: &path::PathBuf) -> path::PathBuf {
    let alt_path = path::PathBuf::from(path);
    let extension = match alt_path.extension() {
        Some(ext) => ext,
        _ => return alt_path,
    };

    let ext_str = match extension.to_str() {
        Some(e) => e,
        _ => return alt_path,
    };

    if !compressed_ext(ext_str) {
        return alt_path;
    }

    match path.file_stem() {
        Some(p) => path::PathBuf::from(p),
        _ => alt_path,
    }
}
