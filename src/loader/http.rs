use crate::loader::{LoadError, ResourceLoader};
use crate::url::Url;
use std::io::Read;

/// An HTTP(S) resource loader using the `ureq` crate.
pub struct HttpLoader;

impl ResourceLoader for HttpLoader {
    fn load(&self, url: &Url) -> Result<Vec<u8>, LoadError> {
        // // spec: support http and https schemes
        if url.scheme != "http" && url.scheme != "https" {
            return Err(LoadError::UnsupportedScheme);
        }

        let url_str = url.serialize();

        // // spec: follow redirects, decode gzip
        // ureq v3 follows redirects and decodes gzip by default if the features are enabled.
        let response = ureq::get(url_str).call().map_err(|e| match e {
            ureq::Error::StatusCode(404) => LoadError::NotFound,
            _ => LoadError::Io(e.to_string()),
        })?;

        let mut body = Vec::new();
        response
            .into_body()
            .into_reader()
            .read_to_end(&mut body)
            .map_err(|e| LoadError::Io(e.to_string()))?;

        Ok(body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_loader_unsupported_scheme() {
        let loader = HttpLoader;
        let url = Url::parse("file:///test.txt").unwrap();
        let result = loader.load(&url);
        assert_eq!(result, Err(LoadError::UnsupportedScheme));
    }

    #[test]
    #[ignore] // CI must pass without network
    fn test_http_loader_real_network() {
        let loader = HttpLoader;
        let url = Url::parse("http://example.com/").unwrap();
        let result = loader.load(&url).unwrap();
        assert!(!result.is_empty());
        assert!(String::from_utf8_lossy(&result).contains("Example Domain"));
    }
}
