mod http;

pub use http::HttpLoader;

use crate::url::Url;
use std::fs;
use std::path::{Path, PathBuf};

/// The HTML `loading` attribute value for resources (img/iframe): a hint for whether the
/// resource may be deferred until near the viewport. Per the HTML spec the attribute is an
/// enumerated attribute whose missing/invalid default maps to eager loading.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LoadingMode {
    #[default]
    Eager,
    Lazy,
}

impl LoadingMode {
    /// Whether the resource fetch may be deferred (true only for `Lazy`).
    pub fn is_deferred(self) -> bool {
        matches!(self, LoadingMode::Lazy)
    }
}

/// Parses the HTML `loading` attribute value. Matching is ASCII-case-insensitive.
/// `lazy` => Lazy; everything else (including absent/empty/unknown) => Eager (spec default).
pub fn parse_loading_attr(value: &str) -> LoadingMode {
    let trimmed = value.trim_matches(crate::ascii::is_html_whitespace);
    if trimmed.eq_ignore_ascii_case("lazy") {
        LoadingMode::Lazy
    } else {
        LoadingMode::Eager
    }
}

/// The HTML `decoding` attribute value for images: a hint for whether the image decoding
/// should be performed synchronously or asynchronously. Per the HTML spec the attribute is an
/// enumerated attribute whose missing/invalid default maps to auto decoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DecodingMode {
    #[default]
    Auto,
    Sync,
    Async,
}

/// Parses the HTML `decoding` attribute value. Matching is ASCII-case-insensitive.
/// `sync` => Sync, `async` => Async, everything else (including absent/empty/unknown) => Auto (spec default).
pub fn parse_decoding_attr(value: &str) -> DecodingMode {
    let trimmed = value.trim_matches(crate::ascii::is_html_whitespace);
    if trimmed.eq_ignore_ascii_case("sync") {
        DecodingMode::Sync
    } else if trimmed.eq_ignore_ascii_case("async") {
        DecodingMode::Async
    } else {
        DecodingMode::Auto
    }
}

/// The outcome of planning/fetching an image resource.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImageLoadOutcome {
    /// The fetch was deferred (loading="lazy"); no I/O was performed.
    Deferred,
    /// The image was loaded eagerly; carries the fetched bytes.
    Loaded(Vec<u8>),
    /// An eager load was attempted but failed (e.g. denied/unreadable/unsupported).
    Failed,
}

/// Plans an image load honoring the HTML `loading` hint.
///
/// Under the current headless rendering pipeline, there is no scroll viewport, so images
/// marked with `loading="lazy"` (represented by `LoadingMode::Lazy`) are fetched immediately,
/// identical to `LoadingMode::Eager`. The `mode` parameter is retained as a hint for future
/// viewport-aware deferral.
///
/// // TODO(spec): true viewport-proximity deferral (returning `ImageLoadOutcome::Deferred`)
/// should be reinstated once scroll and viewport support exists.
pub fn plan_image_load(src: &str, base_url: Option<&Url>, mode: LoadingMode) -> ImageLoadOutcome {
    // Keep mode parameter for future viewport-aware deferral.
    let _ = mode;
    match load_image_safely(src, base_url) {
        Some(bytes) => ImageLoadOutcome::Loaded(bytes),
        None => ImageLoadOutcome::Failed,
    }
}

/// Decodes a simple base64 encoded string.
/// Returns `None` if the input contains invalid base64 characters.
pub fn decode_base64(input: &str) -> Option<Vec<u8>> {
    let mut bytes = Vec::new();
    let mut buffer = 0u32;
    let mut bits = 0;

    for c in input.chars() {
        if c.is_whitespace() || c == '=' {
            continue;
        }

        let val = match c {
            'A'..='Z' => c as u32 - 'A' as u32,
            'a'..='z' => c as u32 - 'a' as u32 + 26,
            '0'..='9' => c as u32 - '0' as u32 + 52,
            '+' | '-' => 62,
            '/' | '_' => 63,
            _ => return None,
        };

        buffer = (buffer << 6) | val;
        bits += 6;

        if bits >= 8 {
            bits -= 8;
            bytes.push((buffer >> bits) as u8);
        }
    }

    Some(bytes)
}

/// Decodes image bytes from a data URI.
/// Supports both base64 and percent-encoded data.
pub fn load_data_uri(src: &str) -> Option<Vec<u8>> {
    if !src.starts_with("data:") {
        return None;
    }
    let comma_idx = src.find(',')?;
    let metadata = &src["data:".len()..comma_idx];
    let payload = &src[comma_idx + 1..];

    if metadata.contains(";base64") {
        decode_base64(payload)
    } else {
        let decoded = crate::url::percent_decode(payload);
        Some(decoded)
    }
}

/// Verification helper for local files. Only allows files located within the current working
/// directory or the temporary directory.
fn is_path_allowed(path: &std::path::Path) -> bool {
    let canonical_path = match std::fs::canonicalize(path) {
        Ok(p) => p,
        Err(_) => return false,
    };

    if let Ok(cwd) = std::env::current_dir()
        && let Ok(cwd_canonical) = std::fs::canonicalize(&cwd)
        && canonical_path.starts_with(cwd_canonical)
    {
        return true;
    }

    let temp_dir = std::env::temp_dir();
    if let Ok(temp_canonical) = std::fs::canonicalize(&temp_dir)
        && canonical_path.starts_with(temp_canonical)
    {
        return true;
    }

    false
}

/// Gated function to fetch image bytes from a source, using the specified document's base URL (if any).
/// Ensures that:
/// - `data:` URIs are parsed and decoded safely.
/// - `http:` and `https:` resources are loaded via HttpLoader.
/// - Local filesystem reads (`file://` and raw paths) are restricted:
///   - Completely DENIED when requested from a remote page (e.g. `base_url` is http(s)).
///   - Checked via `is_path_allowed` to ensure they do not escape the workspace or temporary directories.
///
/// spec: S-65 / F-3
pub fn load_image_safely(src: &str, base_url: Option<&Url>) -> Option<Vec<u8>> {
    if src.starts_with("data:") {
        return load_data_uri(src);
    }

    let resolved_url = if let Some(base) = base_url {
        crate::url::resolve(base, src)
    } else {
        Url::parse(src).ok()
    };

    if let Some(url) = resolved_url {
        if url.scheme == "data" {
            return load_data_uri(&url.serialize());
        }

        if url.scheme == "http" || url.scheme == "https" {
            let loader = HttpLoader;
            if let Ok(bytes) = loader.load(&url) {
                return Some(bytes);
            }
            return None;
        }

        if url.scheme == "file" {
            if let Some(base) = base_url
                && (base.scheme == "http" || base.scheme == "https")
            {
                return None;
            }

            let path_str = url.path.trim_start_matches('/');
            let path = Path::new(path_str);
            if is_path_allowed(path)
                && let Ok(bytes) = fs::read(path)
            {
                return Some(bytes);
            }

            // Fallback for file URLs that might be relative
            let relative_path = Path::new(path_str);
            if is_path_allowed(relative_path)
                && let Ok(bytes) = fs::read(relative_path)
            {
                return Some(bytes);
            }
            return None;
        }
    }

    if let Some(base) = base_url
        && (base.scheme == "http" || base.scheme == "https")
    {
        return None;
    }

    let path = Path::new(src);
    if is_path_allowed(path)
        && let Ok(bytes) = fs::read(path)
    {
        return Some(bytes);
    }

    None
}

/// Error types for resource loading.
#[derive(Debug, PartialEq, Eq)]
pub enum LoadError {
    /// The URL scheme is not supported by this loader.
    UnsupportedScheme,
    /// The requested resource was not found.
    NotFound,
    /// An I/O error occurred during loading.
    Io(String),
    /// The requested path is outside the configured root directory.
    OutsideRoot,
    // TODO(spec): Add TooManyRedirects variant to LoadError when outside match arms (e.g. in src/engine/mod.rs) are updated to have wildcard/wildcard-like defaults.
}

/// A rich response containing the loaded bytes, Content-Type, and charset (if determined).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoaderResponse {
    /// The loaded raw resource bytes.
    pub bytes: Vec<u8>,
    /// The parsed or sniffed Content-Type of the resource (e.g. "text/html").
    pub content_type: String,
    /// The determined charset (e.g. "utf-8"), if available.
    pub charset: Option<String>,
}

/// Metadata representing one HTTP redirect hop.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedirectMeta {
    /// The HTTP status code of this hop.
    pub status: u16,
    /// The raw value of the "Location" header if present.
    pub location: Option<String>,
}

/// The maximum number of redirect hops allowed.
pub const MAX_REDIRECTS: usize = 10;

/// Reusable, generic, network-free redirect-following function.
/// Also returns the final resolved URL that was actually fetched.
pub fn follow_redirects<F>(start: &Url, mut fetch: F) -> Result<(LoaderResponse, Url), LoadError>
where
    F: FnMut(&Url) -> Result<(RedirectMeta, LoaderResponse), LoadError>,
{
    let mut current_url = start.clone();
    let mut redirect_count = 0;

    loop {
        let (meta, resp) = fetch(&current_url)?;
        let is_redirect = matches!(meta.status, 301 | 302 | 303 | 307 | 308);

        if is_redirect && let Some(ref location) = meta.location {
            if redirect_count >= MAX_REDIRECTS {
                // TODO(spec): Return Err(LoadError::TooManyRedirects) once variant is active
                return Err(LoadError::Io("Too many redirects".to_string()));
            }
            if let Some(resolved) = crate::url::resolve(&current_url, location) {
                current_url = resolved;
                redirect_count += 1;
                continue;
            } else {
                return Ok((resp, current_url));
            }
        }

        return Ok((resp, current_url));
    }
}

/// Parses a Content-Type header value.
/// Returns a tuple of (media_type, charset).
/// Both are in lowercase.
/// Example: "text/html; charset=UTF-8" -> ("text/html", Some("utf-8"))
pub fn parse_content_type(header_val: &str) -> (String, Option<String>) {
    let mut parts = header_val.split(';');
    let media_type = parts.next().unwrap_or("").trim().to_ascii_lowercase();

    let mut charset = None;
    for part in parts {
        let part = part.trim();
        if let Some((k, v)) = part.split_once('=')
            && k.trim().eq_ignore_ascii_case("charset")
        {
            let mut val = v.trim().to_ascii_lowercase();
            if val.starts_with('"') && val.ends_with('"') && val.len() >= 2 {
                val = val[1..val.len() - 1].to_string();
            }
            charset = Some(val);
            break;
        }
    }

    (media_type, charset)
}

fn get_first_significant_bytes(bytes: &[u8]) -> &[u8] {
    let mut start = 0;
    // Skip BOMs if present
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        start += 3;
    } else if bytes.starts_with(&[0xFE, 0xFF]) || bytes.starts_with(&[0xFF, 0xFE]) {
        start += 2;
    }
    while start < bytes.len() {
        let b = bytes[start];
        if b == b' ' || b == b'\t' || b == b'\n' || b == b'\r' || b == 0x0C {
            start += 1;
        } else {
            break;
        }
    }
    &bytes[start..]
}

/// Sniffs the content type of the raw resource bytes.
/// Supports HTML, CSS, and basic image types.
pub fn sniff_content_type_from_bytes(bytes: &[u8]) -> Option<String> {
    if bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) {
        return Some("image/png".to_string());
    }
    if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return Some("image/jpeg".to_string());
    }
    if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        return Some("image/gif".to_string());
    }
    if bytes.starts_with(b"RIFF") && bytes.len() >= 12 && &bytes[8..12] == b"WEBP" {
        return Some("image/webp".to_string());
    }

    let sig = get_first_significant_bytes(bytes);
    let limit = std::cmp::min(sig.len(), 100);
    let prefix = &sig[..limit];
    let prefix_lower = prefix.to_ascii_lowercase();

    if prefix_lower.starts_with(b"<!doctype")
        || prefix_lower.starts_with(b"<html")
        || prefix_lower.starts_with(b"<head")
        || prefix_lower.starts_with(b"<body")
        || prefix_lower.starts_with(b"<title")
        || prefix_lower.starts_with(b"<!--")
    {
        return Some("text/html".to_string());
    }

    if prefix_lower.starts_with(b"<?xml") || prefix_lower.starts_with(b"<svg") {
        return Some("image/svg+xml".to_string());
    }

    if prefix_lower.starts_with(b"@charset")
        || prefix_lower.starts_with(b"@import")
        || prefix_lower.starts_with(b"/*")
    {
        return Some("text/css".to_string());
    }

    None
}

/// Determines content type based on the file extension of the URL.
pub fn sniff_content_type_from_extension(url: &Url) -> Option<String> {
    let extension = std::path::Path::new(&url.path)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase());

    match extension.as_deref() {
        Some("html") | Some("htm") => Some("text/html".to_string()),
        Some("css") => Some("text/css".to_string()),
        Some("png") => Some("image/png".to_string()),
        Some("jpg") | Some("jpeg") => Some("image/jpeg".to_string()),
        Some("gif") => Some("image/gif".to_string()),
        Some("webp") => Some("image/webp".to_string()),
        Some("svg") => Some("image/svg+xml".to_string()),
        _ => None,
    }
}

/// Helper to sniff Content-Type and charset from bytes, URL, and transport label.
pub fn sniff_response(
    bytes: &[u8],
    url: &Url,
    transport_content_type: Option<&str>,
) -> (String, Option<String>) {
    // 1. If we have a transport content type, parse it first
    let (mut media_type, mut charset) = if let Some(t_ct) = transport_content_type {
        let (mt, cs) = parse_content_type(t_ct);
        (mt, cs)
    } else {
        (String::new(), None)
    };

    // If media type is generic or empty, try content/extension sniffing
    let is_generic = media_type.is_empty()
        || media_type == "application/octet-stream"
        || media_type == "text/plain";

    if is_generic {
        // A. Content Sniffing (Magic numbers)
        if let Some(sniffed_mt) = sniff_content_type_from_bytes(bytes) {
            media_type = sniffed_mt;
        } else {
            // B. Extension Sniffing
            if let Some(ext_mt) = sniff_content_type_from_extension(url) {
                media_type = ext_mt;
            } else {
                // Default fallback
                media_type = "text/html".to_string();
            }
        }
    }

    // 2. Charset determination
    // If charset is not determined by transport, perform BOM sniffing or meta prescan
    if charset.is_none() {
        let sniffed_charset = crate::encoding::sniff_charset(bytes, None);
        charset = match sniffed_charset {
            crate::encoding::Charset::Utf8 => Some("utf-8".to_string()),
            crate::encoding::Charset::Utf16Le => Some("utf-16le".to_string()),
            crate::encoding::Charset::Utf16Be => Some("utf-16be".to_string()),
            crate::encoding::Charset::Windows1252 => Some("windows-1252".to_string()),
            crate::encoding::Charset::Windows1251 => Some("windows-1251".to_string()),
            crate::encoding::Charset::Windows1250 => Some("windows-1250".to_string()),
            crate::encoding::Charset::Windows1253 => Some("windows-1253".to_string()),
            crate::encoding::Charset::Windows1254 => Some("windows-1254".to_string()),
            crate::encoding::Charset::Windows1255 => Some("windows-1255".to_string()),
            crate::encoding::Charset::Windows1256 => Some("windows-1256".to_string()),
            crate::encoding::Charset::Windows1257 => Some("windows-1257".to_string()),
            crate::encoding::Charset::Windows1258 => Some("windows-1258".to_string()),
            crate::encoding::Charset::Iso8859_15 => Some("iso-8859-15".to_string()),
        };
    }

    (media_type, charset)
}

/// The HTTP method for a resource request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    Get,
    Post,
}

/// A trait for loading resources from a given URL.
pub trait ResourceLoader {
    /// Loads the resource at the specified URL.
    fn load(&self, url: &Url) -> Result<Vec<u8>, LoadError>;

    /// Loads the resource at the specified URL with a richer response containing Content-Type and charset.
    fn load_rich(&self, url: &Url) -> Result<LoaderResponse, LoadError> {
        let bytes = self.load(url)?;
        let (content_type, charset) = sniff_response(&bytes, url, None);
        Ok(LoaderResponse {
            bytes,
            content_type,
            charset,
        })
    }

    /// Performs a request with a method, body, and content-type.
    fn load_request(
        &self,
        url: &Url,
        method: HttpMethod,
        _body: &[u8],
        _content_type: Option<&str>,
    ) -> Result<LoaderResponse, LoadError> {
        match method {
            HttpMethod::Get => self.load_rich(url),
            _ => Err(LoadError::UnsupportedScheme),
        }
    }

    /// Performs a single load "hop", returning HTTP-level redirect metadata
    /// (status code + optional Location header) alongside the response, so that
    /// callers can drive [`follow_redirects`]. The default implementation performs
    /// an ordinary [`ResourceLoader::load_request`] and reports a terminal 200
    /// response with no Location, preserving existing non-redirecting behavior.
    /// Loaders able to surface real HTTP status and headers override this.
    fn load_request_hop(
        &self,
        url: &Url,
        method: HttpMethod,
        body: &[u8],
        content_type: Option<&str>,
    ) -> Result<(RedirectMeta, LoaderResponse), LoadError> {
        let resp = self.load_request(url, method, body, content_type)?;
        Ok((
            RedirectMeta {
                status: 200,
                location: None,
            },
            resp,
        ))
    }
}

/// A filesystem-based resource loader.
pub struct FsLoader {
    root: PathBuf,
}

impl FsLoader {
    /// Creates a new `FsLoader` with the specified root directory.
    pub fn new(root: impl Into<PathBuf>) -> Self {
        let root = root.into();
        // Try to canonicalize the root to ensure consistent path comparisons.
        // If it doesn't exist or can't be canonicalized, use it as is.
        let root = fs::canonicalize(&root).unwrap_or(root);
        Self { root }
    }
}

impl ResourceLoader for FsLoader {
    fn load(&self, url: &Url) -> Result<Vec<u8>, LoadError> {
        // // spec: file: scheme only
        if url.scheme != "file" {
            return Err(LoadError::UnsupportedScheme);
        }

        // url.path is expected to be an absolute-looking path (e.g., "/foo/bar")
        // We treat it as relative to the root.
        let path_str = url.path.trim_start_matches('/');

        // Security check: ensure the path stays within the root.
        // // spec: reject any path that escapes the root via .. or absolute recombination
        let mut target_path = self.root.clone();
        for component in Path::new(path_str).components() {
            match component {
                std::path::Component::Normal(c) => target_path.push(c),
                std::path::Component::ParentDir => {
                    if !target_path.pop() || !target_path.starts_with(&self.root) {
                        return Err(LoadError::OutsideRoot);
                    }
                }
                std::path::Component::RootDir => {
                    // Absolute recombination is not allowed if it points outside root.
                    // Since we already have a root, we treat any absolute path as relative to it,
                    // but if it's explicitly /, we just stay at root.
                }
                _ => {}
            }
        }

        // Final verification with canonicalize to handle symlinks.
        let final_path = match fs::canonicalize(&target_path) {
            Ok(p) => {
                if p.starts_with(&self.root) {
                    p
                } else {
                    return Err(LoadError::OutsideRoot);
                }
            }
            Err(e) => {
                // If the file doesn't exist, we've already checked that it doesn't escape via ..
                // but we still need to check if it's currently outside the root
                // (e.g. if root itself was somehow moved or is weird, though less likely).
                if !target_path.starts_with(&self.root) {
                    return Err(LoadError::OutsideRoot);
                }

                if e.kind() == std::io::ErrorKind::NotFound {
                    return Err(LoadError::NotFound);
                }
                return Err(LoadError::Io(e.to_string()));
            }
        };

        fs::read(final_path).map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => LoadError::NotFound,
            _ => LoadError::Io(e.to_string()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_fs_loader_load_success() {
        let temp_dir = env::temp_dir().join("underrated_loader_test_success");
        fs::create_dir_all(&temp_dir).unwrap();
        let file_path = temp_dir.join("hello.txt");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"Hello, world!").unwrap();

        let loader = FsLoader::new(&temp_dir);
        let url = Url::parse("file:///hello.txt").unwrap();
        let result = loader.load(&url).unwrap();

        assert_eq!(result, b"Hello, world!");
        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_fs_loader_not_found() {
        let temp_dir = env::temp_dir().join("underrated_loader_test_not_found");
        fs::create_dir_all(&temp_dir).unwrap();
        let loader = FsLoader::new(&temp_dir);
        let url = Url::parse("file:///nonexistent.txt").unwrap();
        let result = loader.load(&url);

        assert_eq!(result, Err(LoadError::NotFound));
        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_fs_loader_unsupported_scheme() {
        let loader = FsLoader::new(env::temp_dir());
        let url = Url::parse("https://example.com/").unwrap();
        let result = loader.load(&url);

        assert_eq!(result, Err(LoadError::UnsupportedScheme));
    }

    #[test]
    fn test_fs_loader_outside_root() {
        let temp_dir = env::temp_dir().join("underrated_loader_test_outside");
        fs::create_dir_all(&temp_dir).unwrap();
        let loader = FsLoader::new(&temp_dir);

        // Manually construct a URL that escapes the root to bypass Url::parse normalization.
        let url = Url {
            scheme: "file".to_string(),
            host: None,
            port: None,
            path: "/../../etc/passwd".to_string(),
            query: None,
            fragment: None,
        };
        let result = loader.load(&url);

        assert_eq!(result, Err(LoadError::OutsideRoot));
        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_fs_loader_subdir_success() {
        let temp_dir = env::temp_dir().join("underrated_loader_test_subdir");
        let sub_dir = temp_dir.join("subdir");
        fs::create_dir_all(&sub_dir).unwrap();
        let file_path = sub_dir.join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"subdir content").unwrap();

        let loader = FsLoader::new(&temp_dir);
        let url = Url::parse("file:///subdir/test.txt").unwrap();
        let result = loader.load(&url).unwrap();

        assert_eq!(result, b"subdir content");
        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_decode_base64_basic() {
        assert_eq!(decode_base64("SGVsbG8="), Some(b"Hello".to_vec()));
        assert_eq!(
            decode_base64("SGVsbG8gd29ybGQ="),
            Some(b"Hello world".to_vec())
        );
        assert_eq!(decode_base64("SGVsbG8gd29ybGQ!"), None); // contains literal '!'
    }

    #[test]
    fn test_load_data_uri_base64() {
        let uri = "data:text/plain;base64,SGVsbG8=";
        assert_eq!(load_data_uri(uri), Some(b"Hello".to_vec()));
    }

    #[test]
    fn test_load_data_uri_percent() {
        let uri = "data:text/plain,Hello%20world";
        assert_eq!(load_data_uri(uri), Some(b"Hello world".to_vec()));
    }

    #[test]
    fn test_load_image_safely_deny_etc_passwd() {
        // file:///etc/passwd is always denied
        assert_eq!(load_image_safely("file:///etc/passwd", None), None);
        assert_eq!(load_image_safely("/etc/passwd", None), None);
        assert_eq!(load_image_safely("../../etc/passwd", None), None);
    }

    #[test]
    fn test_load_image_safely_deny_from_remote() {
        let remote_base = Url::parse("https://example.com/").unwrap();
        // local files should be completely denied if base_url is remote http(s)
        assert_eq!(
            load_image_safely("file:///etc/passwd", Some(&remote_base)),
            None
        );
        assert_eq!(load_image_safely("/etc/passwd", Some(&remote_base)), None);
        assert_eq!(
            load_image_safely("temp_test_rasterize_image_blit.png", Some(&remote_base)),
            None
        );
    }

    #[test]
    fn test_load_image_safely_allow_cwd_and_temp() {
        let temp_dir = env::temp_dir();
        let test_file = temp_dir.join("test_load_image_safely_allow.txt");
        fs::write(&test_file, b"allowed_temp_content").unwrap();

        // Must be allowed since it's within the temp dir
        let path_str = test_file.to_str().unwrap();
        assert_eq!(
            load_image_safely(path_str, None),
            Some(b"allowed_temp_content".to_vec())
        );

        let _ = fs::remove_file(test_file);
    }

    #[test]
    fn test_parse_content_type_spec() {
        assert_eq!(
            parse_content_type("text/html;charset=utf-8"),
            ("text/html".to_string(), Some("utf-8".to_string()))
        );
        assert_eq!(
            parse_content_type("TEXT/HTML; charset=\"UTF-8\""),
            ("text/html".to_string(), Some("utf-8".to_string()))
        );
        assert_eq!(
            parse_content_type("text/css"),
            ("text/css".to_string(), None)
        );
    }

    #[test]
    fn test_sniff_content_type_from_bytes_spec() {
        assert_eq!(
            sniff_content_type_from_bytes(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]),
            Some("image/png".to_string())
        );
        assert_eq!(
            sniff_content_type_from_bytes(b"<!DOCTYPE html><html></html>"),
            Some("text/html".to_string())
        );
        // HTML sniffing ignoring leading BOM and spaces
        assert_eq!(
            sniff_content_type_from_bytes(b"\xEF\xBB\xBF  \n <html lang=\"en\">"),
            Some("text/html".to_string())
        );
        assert_eq!(
            sniff_content_type_from_bytes(b"/* CSS Comment */\nbody { margin: 0; }"),
            Some("text/css".to_string())
        );
        assert_eq!(
            sniff_content_type_from_bytes(b"generic random bytes that do not look like html"),
            None
        );
    }

    #[test]
    fn test_sniff_content_type_from_extension_spec() {
        let url_html = Url::parse("file:///index.html").unwrap();
        assert_eq!(
            sniff_content_type_from_extension(&url_html),
            Some("text/html".to_string())
        );

        let url_css = Url::parse("file:///style.css").unwrap();
        assert_eq!(
            sniff_content_type_from_extension(&url_css),
            Some("text/css".to_string())
        );

        let url_unknown = Url::parse("file:///unknown.dat").unwrap();
        assert_eq!(sniff_content_type_from_extension(&url_unknown), None);
    }

    #[test]
    fn test_fs_loader_load_rich_spec() {
        let temp_dir = env::temp_dir().join("underrated_loader_test_rich");
        fs::create_dir_all(&temp_dir).unwrap();

        // 1. Create a file with CSS content but no specific css extension
        let file_path_css = temp_dir.join("style.txt");
        let mut file_css = File::create(&file_path_css).unwrap();
        file_css
            .write_all(b"/* CSS */\nbody { color: red; }")
            .unwrap();

        let loader = FsLoader::new(&temp_dir);
        let url_css = Url::parse("file:///style.txt").unwrap();
        let res_css = loader.load_rich(&url_css).unwrap();

        assert_eq!(res_css.content_type, "text/css");
        assert_eq!(res_css.charset, Some("windows-1252".to_string())); // fallback

        // 2. Create a file with HTML content, UTF-8 BOM, and .html extension
        let file_path_html = temp_dir.join("index.html");
        let mut file_html = File::create(&file_path_html).unwrap();
        file_html
            .write_all(b"\xEF\xBB\xBF<!doctype html>hello")
            .unwrap();

        let url_html = Url::parse("file:///index.html").unwrap();
        let res_html = loader.load_rich(&url_html).unwrap();

        assert_eq!(res_html.content_type, "text/html");
        assert_eq!(res_html.charset, Some("utf-8".to_string())); // BOM detected

        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_follow_redirects_single_302_absolute() {
        let start_url = Url::parse("http://example.com/start").unwrap();

        let mut seen = Vec::new();

        let (result, final_url) = follow_redirects(&start_url, |url| {
            seen.push(url.serialize());
            if url.serialize() == "http://example.com/start" {
                Ok((
                    RedirectMeta {
                        status: 302,
                        location: Some("http://example.com/target".to_string()),
                    },
                    LoaderResponse {
                        bytes: b"Redirecting...".to_vec(),
                        content_type: "text/html".to_string(),
                        charset: Some("utf-8".to_string()),
                    },
                ))
            } else {
                Ok((
                    RedirectMeta {
                        status: 200,
                        location: None,
                    },
                    LoaderResponse {
                        bytes: b"Final Content".to_vec(),
                        content_type: "text/plain".to_string(),
                        charset: Some("utf-8".to_string()),
                    },
                ))
            }
        })
        .unwrap();

        assert_eq!(result.bytes, b"Final Content");
        assert_eq!(result.content_type, "text/plain");
        assert_eq!(final_url.serialize(), "http://example.com/target");
        assert_eq!(
            seen,
            vec![
                "http://example.com/start".to_string(),
                "http://example.com/target".to_string()
            ]
        );
    }

    #[test]
    fn test_follow_redirects_relative() {
        let start_url = Url::parse("http://example.com/search/start").unwrap();

        let mut seen = Vec::new();

        let (result, final_url) = follow_redirects(&start_url, |url| {
            seen.push(url.serialize());
            if url.serialize() == "http://example.com/search/start" {
                Ok((
                    RedirectMeta {
                        status: 302,
                        location: Some("/results?q=x".to_string()),
                    },
                    LoaderResponse {
                        bytes: b"Redirecting...".to_vec(),
                        content_type: "text/html".to_string(),
                        charset: Some("utf-8".to_string()),
                    },
                ))
            } else {
                Ok((
                    RedirectMeta {
                        status: 200,
                        location: None,
                    },
                    LoaderResponse {
                        bytes: b"Search Results".to_vec(),
                        content_type: "text/html".to_string(),
                        charset: Some("utf-8".to_string()),
                    },
                ))
            }
        })
        .unwrap();

        assert_eq!(result.bytes, b"Search Results");
        assert_eq!(final_url.serialize(), "http://example.com/results?q=x");
        assert_eq!(
            seen,
            vec![
                "http://example.com/search/start".to_string(),
                "http://example.com/results?q=x".to_string()
            ]
        );
    }

    #[test]
    fn test_follow_redirects_exceed_max() {
        let start_url = Url::parse("http://example.com/loop").unwrap();

        let mut seen = Vec::new();

        let result = follow_redirects(&start_url, |url| {
            seen.push(url.serialize());
            let next_url = format!("http://example.com/loop{}", seen.len());
            Ok((
                RedirectMeta {
                    status: 302,
                    location: Some(next_url),
                },
                LoaderResponse {
                    bytes: b"Redirecting forever...".to_vec(),
                    content_type: "text/html".to_string(),
                    charset: Some("utf-8".to_string()),
                },
            ))
        });

        assert!(result.is_err());
        let err = result.err().unwrap();
        assert_eq!(err, LoadError::Io("Too many redirects".to_string()));
        assert_eq!(seen.len(), 11);
    }

    #[test]
    fn test_follow_redirects_no_redirect_200() {
        let start_url = Url::parse("http://example.com/ok").unwrap();

        let mut seen = Vec::new();

        let (result, final_url) = follow_redirects(&start_url, |url| {
            seen.push(url.serialize());
            Ok((
                RedirectMeta {
                    status: 200,
                    location: None,
                },
                LoaderResponse {
                    bytes: b"OK".to_vec(),
                    content_type: "text/plain".to_string(),
                    charset: Some("utf-8".to_string()),
                },
            ))
        })
        .unwrap();

        assert_eq!(result.bytes, b"OK");
        assert_eq!(final_url.serialize(), "http://example.com/ok");
        assert_eq!(seen, vec!["http://example.com/ok".to_string()]);
    }

    #[test]
    fn test_follow_redirects_3xx_without_location() {
        let start_url = Url::parse("http://example.com/redir_no_loc").unwrap();

        let mut seen = Vec::new();

        let (result, final_url) = follow_redirects(&start_url, |url| {
            seen.push(url.serialize());
            Ok((
                RedirectMeta {
                    status: 302,
                    location: None,
                },
                LoaderResponse {
                    bytes: b"302 No Location".to_vec(),
                    content_type: "text/html".to_string(),
                    charset: Some("utf-8".to_string()),
                },
            ))
        })
        .unwrap();

        assert_eq!(result.bytes, b"302 No Location");
        assert_eq!(final_url.serialize(), "http://example.com/redir_no_loc");
        assert_eq!(seen, vec!["http://example.com/redir_no_loc".to_string()]);
    }

    #[test]
    fn test_follow_redirects_different_host() {
        let start_url = Url::parse("http://example.com/start").unwrap();

        let mut seen = Vec::new();

        let (result, final_url) = follow_redirects(&start_url, |url| {
            seen.push(url.serialize());
            if url.serialize() == "http://example.com/start" {
                Ok((
                    RedirectMeta {
                        status: 302,
                        location: Some("http://different-host.com/target".to_string()),
                    },
                    LoaderResponse {
                        bytes: b"Redirecting...".to_vec(),
                        content_type: "text/html".to_string(),
                        charset: Some("utf-8".to_string()),
                    },
                ))
            } else {
                Ok((
                    RedirectMeta {
                        status: 200,
                        location: None,
                    },
                    LoaderResponse {
                        bytes: b"Different Host Content".to_vec(),
                        content_type: "text/plain".to_string(),
                        charset: Some("utf-8".to_string()),
                    },
                ))
            }
        })
        .unwrap();

        assert_eq!(result.bytes, b"Different Host Content");
        assert_eq!(final_url.serialize(), "http://different-host.com/target");
        assert_ne!(final_url.serialize(), "http://example.com/start");
        assert_eq!(
            seen,
            vec![
                "http://example.com/start".to_string(),
                "http://different-host.com/target".to_string()
            ]
        );
    }

    struct DefaultMockLoader;

    impl ResourceLoader for DefaultMockLoader {
        fn load(&self, _url: &Url) -> Result<Vec<u8>, LoadError> {
            Ok(b"hello".to_vec())
        }
    }

    #[test]
    fn test_load_request_hop_default_behavior() {
        let loader = DefaultMockLoader;
        let url = Url::parse("http://example.com/test").unwrap();
        let (meta, resp) = loader
            .load_request_hop(&url, HttpMethod::Get, b"", None)
            .unwrap();

        assert_eq!(meta.status, 200);
        assert_eq!(meta.location, None);
        assert_eq!(resp.bytes, b"hello");
    }

    struct OverridingMockLoader;

    impl ResourceLoader for OverridingMockLoader {
        fn load(&self, _url: &Url) -> Result<Vec<u8>, LoadError> {
            Ok(vec![])
        }

        fn load_request_hop(
            &self,
            url: &Url,
            _method: HttpMethod,
            _body: &[u8],
            _content_type: Option<&str>,
        ) -> Result<(RedirectMeta, LoaderResponse), LoadError> {
            if url.serialize() == "http://example.com/start" {
                Ok((
                    RedirectMeta {
                        status: 302,
                        location: Some("/final".to_string()),
                    },
                    LoaderResponse {
                        bytes: b"Redirecting...".to_vec(),
                        content_type: "text/html".to_string(),
                        charset: Some("utf-8".to_string()),
                    },
                ))
            } else if url.serialize() == "http://example.com/final" {
                Ok((
                    RedirectMeta {
                        status: 200,
                        location: None,
                    },
                    LoaderResponse {
                        bytes: b"FINAL".to_vec(),
                        content_type: "text/plain".to_string(),
                        charset: Some("utf-8".to_string()),
                    },
                ))
            } else {
                Err(LoadError::NotFound)
            }
        }
    }

    #[test]
    fn test_load_request_hop_driving_follow_redirects() {
        let loader = OverridingMockLoader;
        let start_url = Url::parse("http://example.com/start").unwrap();

        let (result, final_url) = follow_redirects(&start_url, |u| {
            loader.load_request_hop(u, HttpMethod::Get, b"", None)
        })
        .unwrap();

        assert_eq!(result.bytes, b"FINAL");
        assert_eq!(result.content_type, "text/plain");
        assert_eq!(final_url.serialize(), "http://example.com/final");
    }

    struct NonRedirectMockLoader;

    impl ResourceLoader for NonRedirectMockLoader {
        fn load(&self, _url: &Url) -> Result<Vec<u8>, LoadError> {
            Ok(vec![])
        }

        fn load_request_hop(
            &self,
            _url: &Url,
            _method: HttpMethod,
            _body: &[u8],
            _content_type: Option<&str>,
        ) -> Result<(RedirectMeta, LoaderResponse), LoadError> {
            Ok((
                RedirectMeta {
                    status: 200,
                    location: None,
                },
                LoaderResponse {
                    bytes: b"OK".to_vec(),
                    content_type: "text/plain".to_string(),
                    charset: Some("utf-8".to_string()),
                },
            ))
        }
    }

    #[test]
    fn test_load_request_hop_non_redirect_passes_through() {
        let loader = NonRedirectMockLoader;
        let start_url = Url::parse("http://example.com/any").unwrap();

        let (result, final_url) = follow_redirects(&start_url, |u| {
            loader.load_request_hop(u, HttpMethod::Get, b"", None)
        })
        .unwrap();

        assert_eq!(result.bytes, b"OK");
        assert_eq!(result.content_type, "text/plain");
        assert_eq!(final_url.serialize(), "http://example.com/any");
    }

    #[test]
    fn test_loading_mode_default() {
        assert_eq!(LoadingMode::default(), LoadingMode::Eager);
    }

    #[test]
    fn test_loading_mode_is_deferred() {
        assert!(LoadingMode::Lazy.is_deferred());
        assert!(!LoadingMode::Eager.is_deferred());
    }

    #[test]
    fn test_parse_loading_attr() {
        // 1. "lazy" => Lazy
        assert_eq!(parse_loading_attr("lazy"), LoadingMode::Lazy);

        // 2. Case-insensitivity and whitespace trimming
        assert_eq!(parse_loading_attr("LAZY"), LoadingMode::Lazy);
        assert_eq!(parse_loading_attr("Lazy"), LoadingMode::Lazy);
        assert_eq!(parse_loading_attr("  lazy  "), LoadingMode::Lazy);
        assert_eq!(parse_loading_attr("\t\n lazy\r\x0C "), LoadingMode::Lazy);

        // 3. "eager" => Eager
        assert_eq!(parse_loading_attr("eager"), LoadingMode::Eager);
        assert_eq!(parse_loading_attr("EAGER"), LoadingMode::Eager);
        assert_eq!(parse_loading_attr("  eager  "), LoadingMode::Eager);

        // 4. Missing/invalid defaults to Eager
        assert_eq!(parse_loading_attr(""), LoadingMode::Eager);
        assert_eq!(parse_loading_attr("auto"), LoadingMode::Eager);
        assert_eq!(parse_loading_attr("garbage"), LoadingMode::Eager);
        assert_eq!(parse_loading_attr("lazyx"), LoadingMode::Eager);
        assert_eq!(parse_loading_attr("xlazy"), LoadingMode::Eager);
    }

    #[test]
    fn test_decoding_mode_default() {
        assert_eq!(DecodingMode::default(), DecodingMode::Auto);
    }

    #[test]
    fn test_parse_decoding_attr() {
        // 1. "sync" => Sync
        assert_eq!(parse_decoding_attr("sync"), DecodingMode::Sync);
        assert_eq!(parse_decoding_attr("SYNC"), DecodingMode::Sync);
        assert_eq!(parse_decoding_attr("Sync"), DecodingMode::Sync);
        assert_eq!(parse_decoding_attr("  sync  "), DecodingMode::Sync);
        assert_eq!(parse_decoding_attr("\t\n sync\r\x0C "), DecodingMode::Sync);

        // 2. "async" => Async
        assert_eq!(parse_decoding_attr("async"), DecodingMode::Async);
        assert_eq!(parse_decoding_attr("ASYNC"), DecodingMode::Async);
        assert_eq!(parse_decoding_attr("Async"), DecodingMode::Async);
        assert_eq!(parse_decoding_attr("  async  "), DecodingMode::Async);
        assert_eq!(
            parse_decoding_attr("\t\n async\r\x0C "),
            DecodingMode::Async
        );

        // 3. "auto" => Auto
        assert_eq!(parse_decoding_attr("auto"), DecodingMode::Auto);
        assert_eq!(parse_decoding_attr("AUTO"), DecodingMode::Auto);
        assert_eq!(parse_decoding_attr("Auto"), DecodingMode::Auto);
        assert_eq!(parse_decoding_attr("  auto  "), DecodingMode::Auto);

        // 4. Missing/invalid defaults to Auto
        assert_eq!(parse_decoding_attr(""), DecodingMode::Auto);
        assert_eq!(parse_decoding_attr("garbage"), DecodingMode::Auto);
        assert_eq!(parse_decoding_attr("syncx"), DecodingMode::Auto);
        assert_eq!(parse_decoding_attr("xsync"), DecodingMode::Auto);
    }

    #[test]
    fn test_plan_image_load() {
        // 1. Under eager-now policy, Lazy fetches immediately. For a bogus src, it returns Failed.
        let result_lazy_fail = plan_image_load(
            "http://non-existent-domain-12345.com/test.png",
            None,
            LoadingMode::Lazy,
        );
        assert_eq!(result_lazy_fail, ImageLoadOutcome::Failed);

        // 2. Eager success: valid data: URI returns Loaded
        let data_uri = "data:text/plain;base64,SGVsbG8=";
        let result_eager_success = plan_image_load(data_uri, None, LoadingMode::Eager);
        assert_eq!(
            result_eager_success,
            ImageLoadOutcome::Loaded(b"Hello".to_vec())
        );

        // 3. Lazy success: valid data: URI returns Loaded (mirroring eager)
        let result_lazy_success = plan_image_load(data_uri, None, LoadingMode::Lazy);
        assert_eq!(
            result_lazy_success,
            ImageLoadOutcome::Loaded(b"Hello".to_vec())
        );
        // Show that lazy and eager now produce the SAME outcome for the same source
        assert_eq!(result_lazy_success, result_eager_success);

        // 4. Eager failure: rejected scheme or garbage path returns Failed
        let result_eager_fail =
            plan_image_load("invalid-scheme://something", None, LoadingMode::Eager);
        assert_eq!(result_eager_fail, ImageLoadOutcome::Failed);
    }
}
