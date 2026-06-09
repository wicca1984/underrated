mod http;

pub use http::HttpLoader;

use crate::url::Url;
use std::fs;
use std::path::{Path, PathBuf};

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
}

/// A trait for loading resources from a given URL.
pub trait ResourceLoader {
    /// Loads the resource at the specified URL.
    fn load(&self, url: &Url) -> Result<Vec<u8>, LoadError>;
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
}
