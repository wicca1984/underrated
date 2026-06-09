use crate::url::Url;
use std::fs;
use std::path::{Path, PathBuf};

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
        // // TODO(spec): support http/https schemes in a separate loader or by extending this one.
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
}
