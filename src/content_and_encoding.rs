use std::path;

// A file with no extention is still a textfile.
// Directories would be transformed into a index.html path.

const TXT: &str = "text/plain; charset=utf-8";

pub fn get_content_type(path: &path::PathBuf) -> &str {
    let extension = match path.extension() {
        Some(ext) => ext,
        _ => return "application/octet-stream",
    };

    let ext_str = match extension.to_str() {
        Some(e) => e,
        _ => return "application/octet-stream",
    };

    match ext_str {
        "aac" => "audio/aac",
        "bmp" => "image/bmp",
        "css" => "text/css; charset=utf-8",
        "csv" => "text/csv; charset=utf-8",
        "flac" => "audio/flac",
        "gif" => "image/gif",
        "gz" => "application/gzip",
        "html" => "text/html; charset=utf-8",
        "ico" => "image/vnd.microsoft.icon",
        "jpeg" => "image/jpeg",
        "jpg" => "image/jpeg",
        "js" => "text/javascript; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        "webmanifest" => "application/manifest+json",
        "m3u8" => "application/x-mpegURL",
        "m3u" => "application/x-mpegURL",
        "midi" => "audio/midi",
        "mp3" => "audio/mpeg",
        "mp4" => "video/mp4",
        "mpd" => "application/dash+xml",
        "mpeg" => "video/mpeg",
        "oga" => "audio/ogg",
        "ogv" => "video/ogg",
        "otf" => "font/otf",
        "pdf" => "application/pdf",
        "png" => "image/png",
        "svg" => "image/svg+xml",
        "txt" => "text/plain; charset=utf-8",
        "tiff" => "image/tiff",
        "ts" => "video/MP2T",
        "ttf" => "font/ttf",
        "wasm" => "application/wasm",
        "wav" => "audio/wav",
        "weba" => "weba",
        "webm" => "video/webm",
        "webp" => "image/webp",
        "woff2" => "font/woff2",
        "woff" => "font/woff",
        "xml" => "application/xml; charset=utf-8",
        "zip" => "application/zip",
        _ => "application/octet-stream",
    }
}

pub fn get_encoded_ext(encoding: &str) -> Option<&str> {
    match encoding {
        "gzip" => Some(".gz"),
        "zstd" => Some(".zst"),
        "br" => Some(".br"),
        "deflate" => Some(".zz"),
        _ => None,
    }
}
